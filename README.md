## Pingway

![alt text](logo.png)

## What is Pingway ?
Pingway is a gateway scratch based on pingora ! 
( pingora is a rust library for building fast, reliable and evolvable network services.  )
Pingora https://github.com/cloudflare/pingora

## Featurre Highlights
1. routing forward
2. access log
3. prometheus int counter metrics

## How to use ?
```
pingway -c pingway.yml
```
Daemonize the server
```
pingway -c pingway.yml -d
```
## Arguments (same with pingora)
| Argument      | Effect        | default|
| ------------- |-------------| ----|
| -d, --daemon | Daemonize the server | false |
| -t, --test | Test the server conf and then exit (WIP) | false |
| -c, --conf | The path to the configuration file | empty string |
| -u, --upgrade | This server should gracefully upgrade a running server | false |
