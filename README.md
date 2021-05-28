# FastJob(WIP)

A distributed, lock-free scheduled platform what provides various ways of scheduling, group isolation, high-performance
etc.

# Architecture

# Detailed Designed

## How to schedule?

Scheduler is a core module in FastJob that supports multiple scheduling strategies, e.g. CRON、Fixed Frequency、Fixed
Delayed、API , but we can ignore API because of it will directly startup through interface that client supports, so the
remaining scheduling policies are divided into regular tasks and second-level tasks based on execution frequency.

* **Regular tasks :** Acquire tasks that ready to executed from database,and generate execution record insert into
  database then push it to delayer timer.
* **Second Level tasks :** Server will not be scheduled and executed and will be directly lowered into the worker, so
  worker will schedule and execute.

**How to conduct reliable scheduling?**
> Scheduler can't acquire task due to network latency or other unknown questions, so how do we schedule reliably?
> WAL(Write-AHead Logging) so server will insert a pre-execution record to database then push task to delayer timer, hence if
> the server cashed that not execute task, other server will process it, unless all servers crashed.

## How to balance high-available and high-performance?

The traditional practice is to start all services and find a registry to register their information such as NACOS then
client lookup service through NACOS then completely connect. Through distribute-lock achieved uniqueness of task
scheduling i.e. only be scheduled by one machine at a time, so it can't do high performance, so we need to use the
lock-free mind designing it and support group isolation.

**How are group defined?**
> Aimed at providing accurate scheduling for various departments and lines of business,so FastJob has the concept of AppName.
> An AppName logically corresponds to a set of tasks for an application and physically corresponds to the cluster to which
> the applications is deployed.

**How do you get all the machines in a group to connect to the same server?**
> If just using a sample hash, there are not going to be able to do that because if you have a network problem or if you
> have added server, the result of hash will go on offset, so we can borrow ideas from other components, such as the NameServer
> in RocketMQ. Random chooses a server when client need to connect then server will respond message contains which server the
> AppName should connect to,so we moved the process of finding a specific server from the client to the server.
> If most of the applications under the AppName successfully connect to the same server, but the server goes down,the server
> will move all the application under the AppName to the available server.

**How to synchronize the metadata?**
> The previous answer addresses lock-free scheduling,but introduces new question that how to synchronize metadata between servers?
> **There are two options below**.
> * Synchronize through database and check the heartbeat periodically.
> * Use a consensus algorithm such as Gossip.
>
> I choose the first option,because of this is easier and faster that the database is used to store information such as task execution records.

# To Do List

- [ ] support map-reduce.
- [ ] container task.
- [ ] task tag.
- [x] support log.
- [ ] support metrics, e.g. prometheus etc.
- [ ] support alarm, e.g. dingding etc.
- [ ] support docker startup.
- [ ] support admin.

# Example

```rust

```

# Startup

### Local

```bash
```

### Docker

```bash

```

# License

Licensed under of either of

* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
