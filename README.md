## Pingway

![alt text](logo.png)

## What is Pingway ?
Pingway is [a gateway scratch based on pingora](https://github.com/arczhi/pingway) ! 

## What is Pingora ?
Pingora is a Rust framework to [build fast, reliable and programmable networked systems](https://blog.cloudflare.com/pingora-open-source).



## Feature Highlights
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

## Config File
```
## pingway.yml
# pingora original settings
version: 1
threads: 2
pid_file: ./tmp/pingway.pid
error_log: ./tmp/pingway_err.log
upgrade_sock: ./tmp/pingway.sock

# pingway supplementary settings
port: 6199
access_log: ./tmp/access.log
upstream:
  - uri_prefix: /jk
    addr: "127.0.0.1:9696"
  - uri_prefix: /cloud
    addr: "127.0.0.1:80"
  - uri_prefix: /test
    addr: "www.test.com:80"
  - uri_prefix: /lang
    addr: "127.0.0.1:9099"
prometheus:
  enabled: true
  port: 9091

```
