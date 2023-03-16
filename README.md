# crosswind

`crosswind` is an IPv6 multicast bridge. Given an multicast IP address and the addresses of one or
more other machines also running `crosswind`, it can rebroadcast link-local multicast traffic from
one network to another. This is useful for e.g. having multicast traffic traverse your Tailscale
tailnet (something that is not supported by Tailscale out of the box).

## Using `crosswind` with Tailscale

If you just want to make a multicast address work across your tailnet, use the `tailscale` command:

```sh
cargo run --bin tailscale --\
    --multicast-address <IPv6 multicast address, including port number>
```

(This requires the [Tailscale CLI](https://tailscale.com/kb/1080/cli/) to be installed locally.)

This will forward any multicast traffic received on the given address to each of the machines in
your tailnet. If they are also running crosswind, they will re-broadcast this traffic back out on
the same multicast address.

## Using `crosswind` directly

For other use cases or more fine-grained control, you can run crosswind directly:

```sh
cargo run --\
    --interface <interface name or local IP address>\
    --multicast-address <IPv6 multicast address, including port number>\
    --targets <IPv6 address + port number of other crosswind instance>\
    --targets <IPv6 address + port number of yet other crosswind instance>\
    --targets <...and so on>\
    --port <port number to listen on; default is 9908>
```
