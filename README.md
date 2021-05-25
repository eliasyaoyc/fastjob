# FastJob

A distributed, lock-free scheduled platform what provides various ways of scheduling, resource isolation, high
performance etc.

# Architecture

# Detailed Designed
## Scheduler Module

Scheduler is a core module in FastJob that has two policies to delivery job to delaytimer. The job that interval of
execution less than 6 seconds that will cached in the server local through grpc, otherwise，fetch the job that to be executed
from the database at regular intervals every five seconds through job-fetcher thread.

## High Available Module
Initial Plan, when server startup will actively query the database cluster message and insert its own metadata data.  
Drawbacks that increase database pressure,so i wonder if it is possible to synchronize metadata through a consensus protocol(e.g. Gossip),
but this is difficult and heavy workload.

# Example

```rust

```

# Startup

### Local

```bash
```

### Cluster

```bash

```

# License
