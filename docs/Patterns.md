
* in js
    * Request-Response Pattern [^Request_Response_Pattern]
    * Command Pattern [^Command_Pattern]
    * Event Bus Pattern [^Event_Bus_Pattern]
    * Single Worker Pattern [^Single_Worker_Pattern]
    * Dedicated Worker Pattern [^Dedicated_Worker_Pattern]



[^Request_Response_Pattern]: Send a message with an identifier (ID); worker responds with the same ID. Allows matching responses to requests.
[^Command_Pattern]: Send messages as commands; worker switches behavior based on command.
[^Event_Bus_Pattern]: Use a central event handler or custom event emitter to route incoming messages to the right listeners.
[^Single_Worker_Pattern]: Main thread creates one worker for offloading specific tasks to. Helps balance CPU load when many small tasks need parallelism.
[^Dedicated_Worker_Pattern]: Each worker is dedicated to one task or module; main thread knows exactly which worker does what.


