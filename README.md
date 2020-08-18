[![stability-experimental](https://img.shields.io/badge/stability-experimental-orange.svg)](#experimental)

# throttled

Simple IOPS and throughput monitoring for Azure VMs. Discovers all disk
and machine limits through instance metadata and exposes current
utilization as a percentage of total uncached limits. Note that ratios
may be greater than 1 with caching enabled, and may briefly peak above 1
before encountering throttling.

## Experimental

Code is new and may change or be removed in future versions. Please try
it out and provide feedback. Feel free to open issues for discussion,
bugs, or desired functionality.


## Log output

The daemon logs json at startup containing the vm and sku limits for
each detected disk.

```
{
  "virtual_machine": {
    "name": "Standard_D8s_v3",
    "location": "westus2",
    "max_iops": 12800,
    "max_bandwidth": 201326592
  },
  "disks": {
    "/dev/sdc": {
      "location": "westus2",
      "storage_account_type": "Premium_LRS",
      "min_size_gb": 64,
      "max_size_gb": 128,
      "max_iops": 500,
      "max_bandwidth": 104857600
    },
    "/dev/sda": {
      "location": "westus2",
      "storage_account_type": "Premium_LRS",
      "min_size_gb": 512,
      "max_size_gb": 1024,
      "max_iops": 5000,
      "max_bandwidth": 209715200
    },
    "/dev/sdd": {
      "location": "westus2",
      "storage_account_type": "Premium_LRS",
      "min_size_gb": 256,
      "max_size_gb": 512,
      "max_iops": 2300,
      "max_bandwidth": 157286400
    }
  }
}
```

## Prometheus metrics

Note that IOPS and bandwidth as raw numbers are available through many
sources, they are here only as a sanity check against the ratios since
they need to be collected anyway for calculation. It's possible to only
expose the limits as fully static metrics to optimize resource usage
(not currently implemented).

In this example, a workload (fio) issues many small writes to the
temporary disk of the VM. We see 8268 IOPS against /dev/sdb (the
temporary disk), and a few IOPS against the /dev/sda, OS disk.

The VM SKU is a Standard_D8s_v3 which has 12k uncache IOPS and 16k
cached IOPS. We see it's at about 64% of its uncached limit.

```
# HELP iops Gauge counting point-in-time IOPS for a given device or host
# TYPE iops gauge
iops{device="/dev/loop0"} 0
iops{device="/dev/sda"} 1.5
iops{device="/dev/sdb"} 8268
iops{device="/dev/sdc"} 0
iops{device="/dev/sdd"} 0
iops{device="TOTAL"} 8269.5

# HELP iops_ratio Gauge representing percentage of iops limit saturated for a given device or host. 1 means 100% of the uncached sku limit is being utilized. This value can be greater than 1 when the sku bursts or uses caching.
# TYPE iops_ratio gauge
iops_ratio{device="/dev/sda"} 0.0003
iops_ratio{device="/dev/sdc"} 0
iops_ratio{device="/dev/sdd"} 0
iops_ratio{device="TOTAL"} 0.646055

# HELP throughput_bytes Gauge counting point-in-time throughput in bytes for a given device or host
# TYPE throughput gauge
throughput_bytes{device="/dev/loop0"} 0
throughput_bytes{device="/dev/sda"} 10
throughput_bytes{device="/dev/sdb"} 33072
throughput_bytes{device="/dev/sdc"} 0
throughput_bytes{device="/dev/sdd"} 0
throughput_bytes{device="TOTAL"} 33082

# HELP throughput_ratio Gauge representing percentage of bandwidth limit saturated by for a given device or host. 1 means 100% of the uncached sku limit is being utilized. This value can be greater than 1 when the sku bursts or uses caching.
# TYPE throughput_ratio gauge
throughput_ratio{device="/dev/sda"} 0
throughput_ratio{device="/dev/sdc"} 0
throughput_ratio{device="/dev/sdd"} 0
throughput_ratio{device="TOTAL"} 0.000164
```

In the next example, the VM is at its uncached limits and aggressively
uses the Azure disk caching layer to achieve the cached throughput
rates, typically about 20-25% higher than the uncached limits. Notice
iops_ratio{device="TOTAL"} is greater than 1.0. In fact, it's about 25%
greater because the VM is at the cached limit (which is typically a hard
cap).

```
# HELP iops Gauge counting point-in-time IOPS for a given device or host
# TYPE iops gauge
iops{device="/dev/loop0"} 0
iops{device="/dev/sda"} 0
iops{device="/dev/sdb"} 16354
iops{device="/dev/sdc"} 0
iops{device="/dev/sdd"} 0
iops{device="TOTAL"} 16354

# HELP iops_ratio Gauge representing percentage of iops limit saturated for a given device or host. 1 means 100% of the uncached sku limit is being utilized. This value can be greater than 1 when the sku bursts or uses caching.
# TYPE iops_ratio gauge
iops_ratio{device="/dev/sda"} 0
iops_ratio{device="/dev/sdc"} 0
iops_ratio{device="/dev/sdd"} 0
iops_ratio{device="TOTAL"} 1.277656

# HELP throughput_bytes Gauge counting point-in-time throughput in bytes for a given device or host
# TYPE throughput_bytes gauge
throughput_bytes{device="/dev/loop0"} 0
throughput_bytes{device="/dev/sda"} 0
throughput_bytes{device="/dev/sdb"} 65416
throughput_bytes{device="/dev/sdc"} 0
throughput_bytes{device="/dev/sdd"} 0
throughput_bytes{device="TOTAL"} 65416

# HELP throughput_ratio Gauge representing percentage of bandwidth limit saturated by for a given device or host. 1 means 100% of the uncached sku limit is being utilized. This value can be greater than 1 when the sku bursts or uses caching.
# TYPE throughput_ratio gauge
throughput_ratio{device="/dev/sda"} 0
throughput_ratio{device="/dev/sdc"} 0
throughput_ratio{device="/dev/sdd"} 0
throughput_ratio{device="TOTAL"} 0.000325
```

In this last example, we see the OS disk burst above its sku limits to
~320% of its limit to saturate the VM limit. The VM limit in this case
is 16k IOPS on a Standard_D8s_v3 while the disk limit is 5k IOPS for a P30.

```
# HELP iops Gauge counting point-in-time IOPS for a given device or host
# TYPE iops gauge
iops{device="/dev/loop0"} 0
iops{device="/dev/sda"} 16432
iops{device="/dev/sdb"} 0
iops{device="/dev/sdc"} 0
iops{device="/dev/sdd"} 0
iops{device="TOTAL"} 16432

# HELP iops_ratio Gauge representing percentage of iops limit saturated for a given device or host. 1 means 100% of the uncached sku limit is being utilized. This value can be greater than 1 when the sku bursts or uses caching.
# TYPE iops_ratio gauge
iops_ratio{device="/dev/sda"} 3.2864
iops_ratio{device="/dev/sdc"} 0
iops_ratio{device="/dev/sdd"} 0
iops_ratio{device="TOTAL"} 1.28375

# HELP throughput_bytes Gauge counting point-in-time throughput in bytes for a given device or host
# TYPE throughput_bytes gauge
throughput_bytes{device="/dev/loop0"} 0
throughput_bytes{device="/dev/sda"} 65728
throughput_bytes{device="/dev/sdb"} 0
throughput_bytes{device="/dev/sdc"} 0
throughput_bytes{device="/dev/sdd"} 0
throughput_bytes{device="TOTAL"} 65728

# HELP throughput_ratio Gauge representing percentage of bandwidth limit saturated by for a given device or host. 1 means 100% of the uncached sku limit is being utilized. This value can be greater than 1 when the sku bursts or uses caching.
# TYPE throughput_ratio gauge
throughput_ratio{device="/dev/sda"} 0.000313
throughput_ratio{device="/dev/sdc"} 0
throughput_ratio{device="/dev/sdd"} 0
throughput_ratio{device="TOTAL"} 0.000326
```

## what it does

The daemon uses IMDS to identify the VM size, OS disk size, and OS disk
storage type. IMDS also provides the size, LUN, and storage types for
all data disks. The SKUs API provides all virtual machine and
disk SKUs with cabilities such as IOPS/bandwidth. Currently, we require
MSI on the VM to authenticate to the SKUs API.

By joining IMDS and SKU data on VM size and storage type, we can
identify VM-level IOPS and bandwidth limits.

By joining IMDS and SKU data on disk size and storage type, we can
identify disk-level IOPS and bandwidth limits.

We use the LUNs from data disks to map to friendly udev labels like /dev/sdX.
This is preferable for metrics capture.

We assume /dev/sda and /dev/sdb are always the OS disk and temporary
disk for linux VMs. This is used only in its absence: We assume iostat
will collect this device and don't manually try to map it using udev rules.

After discovering all limits, the daemon polls iostat (or any similar
method) periodically. It exposes a metrics endpoint Prometheus-style
with the hostname and device name, along with a gauge equal to the
percent of IOPS or bandwidth limit. We use friendly udev labels mapped
from LUNs to attach to each metric.

## Development

The main development dependencies are cargo and docker. A skaffold
configuration exists for testing against an existing Azure cluster.

build:
```
cargo build
```

docker build:
```
docker build . -t yourtag
```

continous development with skaffold:
```
skaffold dev
```

# Contributing

This project welcomes contributions and suggestions.  Most contributions require you to agree to a
Contributor License Agreement (CLA) declaring that you have the right to, and actually do, grant us
the rights to use your contribution. For details, visit https://cla.opensource.microsoft.com.

When you submit a pull request, a CLA bot will automatically determine whether you need to provide
a CLA and decorate the PR appropriately (e.g., status check, comment). Simply follow the instructions
provided by the bot. You will only need to do this once across all repos using our CLA.

This project has adopted the [Microsoft Open Source Code of Conduct](https://opensource.microsoft.com/codeofconduct/).
For more information see the [Code of Conduct FAQ](https://opensource.microsoft.com/codeofconduct/faq/) or
contact [opencode@microsoft.com](mailto:opencode@microsoft.com) with any additional questions or comments.
