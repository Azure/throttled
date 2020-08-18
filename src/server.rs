use eyre::Result;
use futures::prelude::*;
use futures::task::Poll;
use hyper::service::Service;
use hyper::{Body, Request, Response};
use prometheus::{Encoder, TextEncoder};
use std::collections::HashMap;
use std::pin::Pin;

use super::iostat;
use super::metrics;
use super::resource::{Disk, VirtualMachine};

pub struct MetricService {
    pub metrics: metrics::Tracker,
    pub vm_limit: VirtualMachine,
    pub disk_limits: HashMap<String, Disk>,
}

impl<T> Service<T> for MetricService {
    type Response = MetricHandler;
    type Error = std::io::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut futures::task::Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: T) -> Self::Future {
        let metrics = self.metrics.clone();
        let vm_limit = self.vm_limit.clone();
        let disk_limits = self.disk_limits.clone();
        let fut = async move {
            Ok(MetricHandler {
                metrics,
                vm_limit,
                disk_limits,
            })
        };
        Box::pin(fut)
    }
}

pub struct MetricHandler {
    metrics: metrics::Tracker,
    vm_limit: VirtualMachine,
    disk_limits: HashMap<String, Disk>,
}

impl Service<Request<Body>> for MetricHandler {
    type Response = Response<Body>;
    type Error = http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut futures::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: Request<Body>) -> Self::Future {
        // collect and record fresh metrics
        let result = collect(&mut self.metrics, &self.vm_limit, &self.disk_limits);

        if let Err(e) = result {
            let response = Response::new(Body::from(format!("failed to collect metrics: {}", &e)));
            let fut = async { Ok(response) };
            return Box::pin(fut);
        }

        // gather all recorded metrics.
        let mut buffer = vec![];
        let encoder = TextEncoder::new();
        let metric_families = self.metrics.registry.gather();

        if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
            let response = Response::new(Body::from(format!("failed to collect metrics: {}", &e)));
            let fut = async { Ok(response) };
            return Box::pin(fut);
        }

        let response = Response::new(Body::from(buffer));
        let fut = async { Ok(response) };
        Box::pin(fut)
    }
}

fn collect(
    metrics: &mut metrics::Tracker,
    vm_limit: &VirtualMachine,
    disk_limits: &HashMap<String, Disk>,
) -> Result<()> {
    let mut total_iops: f64 = 0.0;
    let mut total_throughput: f64 = 0.0;
    for (disk, stats) in iostat::new()?.iter() {
        let iops = stats.iops();
        let throughput = stats.throughput();

        total_iops += iops;
        total_throughput += throughput;

        metrics.set_iops(disk, stats.iops());
        metrics.set_throughput(disk, stats.throughput());

        let disk_limit = disk_limits.get(disk);
        if disk_limit.is_none() {
            println!("no sku information stored for disk: {}", &disk);
            continue;
        }
        let disk_limit = disk_limit.unwrap();

        let iops_ratio = normalize(iops, disk_limit.max_iops as f64, 6);
        let throughput_ratio = normalize(throughput, disk_limit.max_bandwidth as f64, 6);

        metrics.set_iops_ratio(disk, iops_ratio);
        metrics.set_throughput_ratio(disk, throughput_ratio);
    }

    let total_iops_ratio = normalize(total_iops, vm_limit.max_iops as f64, 6);
    let total_throughput_ratio = normalize(total_throughput, vm_limit.max_bandwidth as f64, 6);

    metrics.set_iops("TOTAL", total_iops);
    metrics.set_throughput("TOTAL", total_throughput);
    metrics.set_iops_ratio("TOTAL", total_iops_ratio);
    metrics.set_throughput_ratio("TOTAL", total_throughput_ratio);

    Ok(())
}

fn normalize(numerator: f64, denominator: f64, accuracy: i32) -> f64 {
    let coefficient = 10_f64.powi(accuracy);
    (coefficient * numerator / denominator).round() / coefficient
}
