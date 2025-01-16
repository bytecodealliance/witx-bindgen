# Native async design

## With the canonical ABI

Imported asynchronous functions take two arguments: A pointer to the argument buffer and a pointer to
the result buffer. They return an i32, the two highest bits indicate the state of the async call, 
the lower bits contain the task id to wait for completion. The argument buffer is freed by the callee.

Exported asynchronous function receive their arguments normally in registers, the return value is set via 
"[task-return]name" and it returns either null or a pointer to the state to pass to the callback
once the blocking task completes.

## Proposal for native

Symmetric asynchronous functions receive their arguments normally, if they return a value they pass
a return value buffer and they return a pointer to the object (EventSubscription) to wait on 
or null (completed).

If the bottommost bit of the return value is set (objects have an even address) the call wasn't started (backpressure) and should be retried once the returned event gets active.

This combines the most efficient parts of the import and export (minimizing allocations and calls).

## Executor

See the crates/symmetric_executor directory. The main functions are create_timer, create_event, subscribe,
register_callback and run.
