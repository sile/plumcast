//! [Prometheus][prometheus] metrics.
//!
//! Note that you can also use [fibers_rpc's metrics] in addition to the metrics defined in this module.
//!
//! [prometheus]: https://prometheus.io/
//! [fibers_rpc's metrics]: https://docs.rs/fibers_rpc/0.2/fibers_rpc/metrics/index.html
use prometrics::metrics::{Counter, MetricBuilder};

/// Metrics of [`Service`].
///
/// [`Service`]: ../struct.Service.html
#[derive(Debug, Clone)]
pub struct ServiceMetrics {
    pub(crate) registered_nodes: Counter,
    pub(crate) deregistered_nodes: Counter,
    pub(crate) destination_unknown_messages: Counter,
}
impl ServiceMetrics {
    /// Metric: `plumcast_service_registered_nodes_total <COUNTER>`
    pub fn registered_nodes(&self) -> u64 {
        self.registered_nodes.value() as u64
    }

    /// Metric: `plumcast_service_deregistered_nodes_total <COUNTER>`
    pub fn deregistered_nodes(&self) -> u64 {
        self.deregistered_nodes.value() as u64
    }

    /// Metric: `plumcast_service_destination_unknown_messages_total <COUNTER>`
    pub fn destination_unknown_messages(&self) -> u64 {
        self.destination_unknown_messages.value() as u64
    }

    pub(crate) fn new(mut builder: MetricBuilder) -> Self {
        builder.namespace("plumcast").subsystem("service");
        ServiceMetrics {
            registered_nodes: builder
                .counter("registered_nodes_total")
                .help("Number of nodes registered so far")
                .finish()
                .expect("Never fails"),
            deregistered_nodes: builder
                .counter("deregistered_nodes_total")
                .help("Number of nodes deregistered so far")
                .finish()
                .expect("Never fails"),
            destination_unknown_messages: builder
                .counter("destination_unknown_messages_total")
                .help("Number of RPC messages received but the destination node is missing")
                .finish()
                .expect("Never fails"),
        }
    }
}
