actor model in rust using tokio


Browser will not create two independent connections to same host, it serializes the requests. 
Try with browser + curl, or two curl commands you should see concurrent behavior.

