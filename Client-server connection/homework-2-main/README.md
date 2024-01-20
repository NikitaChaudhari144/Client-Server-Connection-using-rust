# CSE411 - Advanced Programming Techniques

# Homework 2

**Due Date: 10/5/2023 EOD**

## Learning Outcomes:

- Create a key-value store server that accepts requests to access a centralized data storage over the network
- Create a key-value store client that makes requests to access key-value store over the network
- Test the client/server programs

## Description

Key/value stores play a significant role in modern distributed systems.  They  allow a program to easily store data to a high-availability server, and to easily retrieve that data.  Examples of  key/value stores include Redis, MongoDB, Memcached, DynamoDB, Firebase, CouchBase, Voldemort, and many more.

A key/value store must provide the same interface that a Rust map would offer:

get(key) returns the value associated with key, if present, or null otherwise
del(key)  removes key and its associated value from the store, and returns true.  If the key wasn't in the store, returns false.
put(key, value)  adds a mapping from key to value, if no prior mapping for key exists. If a mapping exists, then it is modified, so that the mapping from key will be to the provided value.

Note that the key/value store must be highly concurrent.  That is, it should use as many threads as are available, and should allow operations on different data elements to proceed simultaneously as much as possible.
In order to be as general as possible, key/value stores typically treat the key as a string, and the value as a byte array.  If there is any structure to the values, the key/value store does not know  about it.
 
To implement the key/value store data structure, you cannot simply use the std::unordered_map, because it does not support concurrency.  You can create your own data structure or use the TBB library for concurrent containers.  If you opt for creating your own data structure, the easiest approach is to create a large array, where each array element holds a linked list. You can use the built-in Rust hash function to hash a key to one of these  lists, and you can achieve concurrency by locking at the granularity of lists.    

Finally, it's important to specify the protocol to be used for sending requests to the key/value store over the network, and for receiving results from the key/value store.  
The format will be similar to the HTTP protocol:  we will make use of newlines and lines of text for metadata, and variable-length content for binary data.  Note: it is acceptable to forbid  newlines within keys.  However, a value may contain arbitrary binary data, to include \0 and \n characters.

### Format of the put request

```
first line: PUT\n
second line: KEY-LEN: number\n
third line: the key, ending with \n
fourth line: VAL-LEN: number\n
remaining lines: the value (VAL-LEN bytes)
```  
### Format of the get request

```
first line: GET\n
second line: KEY-LEN: number\n
third line: the key, ending with \n
```

### Format of the del request

```
first line: DEL\n
second line: KEY-LEN: number\n
third line: the key, ending with \n
```

### Format of the response to GET

```
first line: either OK\n or ERROR\n
second line: RESULT-LEN: number\n
remaining lines: the result value (RESULT-LEN bytes)
```
### Format of the response to PUT and DEL

```
first line: either OK\n or ERROR\n
```
 
 Note: when ERROR is returned, there should be no further data transmitted in the response.
 You should always check for properly-formed input.  
 
You are expected to design and implement two components:

A client that builds key/value requests and sends them over the network to the key/value server. The client should also receive a response and display the results received from the server.
A server that accepts key/value requests from clients, executes the requested operations on the map data structure, and builds a response that is sent to the client. The server may process more than one request from different clients. Be sure to include multithreading in the server implementation.

Both components should use the port 1895 for the network communication.

You also need to test these two components by running four clients and one server. Create test cases for all the possible operations and their outcomes.
 	
Grade distribution

- Server Rust code					30%
- Client Rust code				    30%
- Test cases and Testing				10%
- Report                              20%
