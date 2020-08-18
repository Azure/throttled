use eyre::Result;
use prometheus::{GaugeVec, Opts, Registry};

#[derive(Clone)]
pub struct Tracker {
    pub registry: Registry,
    pub iops_ratio_gauge: GaugeVec,
    pub throughput_ratio_gauge: GaugeVec,
    pub iops_gauge: GaugeVec,
    pub throughput_gauge: GaugeVec,
}

impl Tracker {
    pub fn new() -> Result<Self> {
        let registry = Registry::new();

        let labels = ["device"];

        let iops_ratio_gauge_opts = Opts::new(
            "iops_ratio",
            "Gauge representing percentage of iops limit saturated for a given device or host. \
            1 means 100% of the uncached sku limit is being utilized. \
            This value can be greater than 1 when the sku bursts or uses caching.\
            ",
        );

        let throughput_ratios_opts = Opts::new(
            "throughput_ratio",
            "Gauge representing percentage of bandwidth limit saturated by for a given device or host. \
            1 means 100% of the uncached sku limit is being utilized. \
            This value can be greater than 1 when the sku bursts or uses caching.\
            ",
        );

        let iops_gauge_opts = Opts::new(
            "iops",
            "Gauge counting point-in-time IOPS for a given device or host",
        );

        let throughput_opts = Opts::new(
            "throughput_bytes",
            "Gauge counting point-in-time throughput in bytes for a given device or host",
        );

        let iops_ratio_gauge = GaugeVec::new(iops_ratio_gauge_opts, &labels)?;
        let throughput_ratio_gauge = GaugeVec::new(throughput_ratios_opts, &labels)?;
        let iops_gauge = GaugeVec::new(iops_gauge_opts, &labels)?;
        let throughput_gauge = GaugeVec::new(throughput_opts, &labels)?;

        registry.register(Box::new(iops_gauge.clone()))?;
        registry.register(Box::new(throughput_gauge.clone()))?;
        registry.register(Box::new(iops_ratio_gauge.clone()))?;
        registry.register(Box::new(throughput_ratio_gauge.clone()))?;

        Ok(Self {
            registry,
            iops_ratio_gauge,
            throughput_ratio_gauge,
            iops_gauge,
            throughput_gauge,
        })
    }

    pub fn set_iops(&self, label: &str, value: f64) {
        self.iops_gauge.with_label_values(&[label]).set(value)
    }

    pub fn set_iops_ratio(&self, label: &str, value: f64) {
        self.iops_ratio_gauge.with_label_values(&[label]).set(value)
    }

    pub fn set_throughput(&self, label: &str, value: f64) {
        self.throughput_gauge.with_label_values(&[label]).set(value)
    }

    pub fn set_throughput_ratio(&self, label: &str, value: f64) {
        self.throughput_ratio_gauge
            .with_label_values(&[label])
            .set(value)
    }
}
