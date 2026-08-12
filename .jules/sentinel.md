
## 2024-08-12 - [Denial of Service/Thread Starvation]
**Vulnerability:** N+1 credential resolution within queue-processing loops (e.g. `ssh_worker`, `ftp_worker`) where blocking I/O calls to the database secret store block the async executor thread for each item in the queue.
**Learning:** Blocking operations inside `while` loops processing potentially large batches of items can starve the thread pool, especially when processing items from queues in ECS systems.
**Prevention:** Extract connection resolution to happen once per node iteration outside of the ticket processing loop.
