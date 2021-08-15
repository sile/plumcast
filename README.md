plumcast
========

[![plumcast](https://img.shields.io/crates/v/plumcast.svg)](https://crates.io/crates/plumcast)
[![Documentation](https://docs.rs/plumcast/badge.svg)](https://docs.rs/plumcast)
[![Build Status](https://travis-ci.org/sile/plumcast.svg?branch=master)](https://travis-ci.org/sile/plumcast)
[![Code Coverage](https://codecov.io/gh/sile/plumcast/branch/master/graph/badge.svg)](https://codecov.io/gh/sile/plumcast/branch/master)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A message broadcasting library based on the Plumtree/HyParView algorithms.

[Documentation](https://docs.rs/plumcast)


Properties
----------

### Pros

- Nearly optimal message transmitting count
  - Usually messages are broadcasted via a spanning tree
  - Only the nodes interested in the same messages belong to the same cluster
- Scalable
  - Theoretically, it can handle ten-thousand of nodes or more
- High fault tolerance
  - Spanning trees are automatically repaired if there are crashed nodes
- Dynamic membership
  - Nodes can be added to (removed from) a cluster at any time


### Cons

- No strong guarantee about connectivity of the nodes in a cluster
- No strong guarantee about delivery count of a message
- No guarantee about messages delivery order

If some of the above guarantees are mandatory for your application,
it is need to be provided by upper layers.


References
----------

- [HyParView: a membership protocol for reliable gossip-based broadcast][HyParView]
- [Plumtree: Epidemic Broadcast Trees][Plumtree]

[HyParView]: http://asc.di.fct.unl.pt/~jleitao/pdf/dsn07-leitao.pdf
[Plumtree]: http://www.gsd.inesc-id.pt/~ler/reports/srds07.pdf
