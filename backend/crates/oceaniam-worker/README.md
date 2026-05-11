# OceanIAM Worker

Background worker runtime for OceanIAM.

Provides a cron-based scheduler that discovers workers via [`linkme::distributed_slice`].

Workers are registered from anywhere in the crate graph and run on a configurable interval without a central registry — just implement the [`Worker`] trait and tag it with `#[distributed_slice]`.
