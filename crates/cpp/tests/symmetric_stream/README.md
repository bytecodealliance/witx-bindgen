# Native stream design

A stream has the following members: 

 - read_ready_event_send: the sending side of the read ready (data written, ready_size contains the number of elements)
 - write_ready_event_send: the sending side of the write ready (addr and size are set)
 - read_addr: the address of the registered buffer (valid on write_ready)
 - read_size: maximum number of elements in the buffer (valid on write_ready)
 - ready_size: valid number of elements in the buffer (valid on read_ready)
 - active_instances: Number of references to the stream object, decreased by 

## Special values of ready

 - CLOSED: MIN (0x8000…) also EOF
 - BLOCKED: -1 (normal)
 - CANCELLED: 0 (TBD)

## Sequence

"take" means swap with idle value (read_addr=0, read_size=0, ready=-1)

### Read

 - if ready_size is CLOSED: End of file, ready_size should be BLOCKED
 - if read_addr is set wait for read_ready event
 - write addr and size
 - activate write_ready
 - wait for read_ready
 - take ready_size and process data

### Write

 - (only initally) on EOF set ready_size to CLOSED
 - wait for write_ready
 - on EOF set ready_size to CLOSED
 - assert addr and size is valid, ready is MIN (blocking)
    - addr zero likely EOF (reader closed)
 - take addr and size, write data to buffer
 - store number of valid elements in ready_size
 - activate read_ready

## Functions

A vtable is no longer necessary, but some functions enable shared implementations (perhaps interface by WIT?)

 - create_stream: Create a new stream object
 - start_reading: Register buffer and send event
 - start_writing: Take and return the buffer
 - finish_writing: (can also set eof independently of start_write) set available
   amount and trigger event
 - read_amount: Take and return the number of valid elements
 - read/write_ready_event: Sending side of the event
 - is_ready_to_write: Whether a write needs to wait for write_ready
 - is_write_closed: Whether the write side closed
 - close_read/write: Close one side of the stream

### Open questions

 - how to cancel a read?
   - simply replace addr and size with zero? 
     If already written nothing to do. How to handle race conditions when
     destroying the buffer after cancel? Perhaps a roundtrip to close 
     would be helpful. (activate write_ready, wait read_ready, check for EOF)
   - add a write_busy flag? If addr is zero and ready BLOCKED, then wait for
     ready
 - how to cancel a write?
   - simply flag EOF and activate read_ready
 - Is a future the same data structure?
   - read_size would always be one, ready_size up to one, 
     finish_writing and read_amount could directly close the side
