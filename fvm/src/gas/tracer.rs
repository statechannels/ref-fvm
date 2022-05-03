use std::collections::LinkedList;
use std::time::Duration;

use cid::Cid;
use fvm_shared::MethodNum;
use minstant::Instant;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct GasTrace {
    #[serde(skip)]
    previous: Instant,
    #[serde(skip)]
    cum: Duration,
    pub spans: LinkedList<GasSpan>,
}

impl GasTrace {
    pub fn start() -> GasTrace {
        GasTrace {
            previous: Instant::now(),
            cum: Default::default(),
            spans: Default::default(),
        }
    }

    pub fn finish(self) -> LinkedList<GasSpan> {
        self.spans
    }

    pub fn record(&mut self, context: Context, point: Point, consumption: Consumption) {
        let now = Instant::now();
        let elapsed_rel = now - self.previous;

        self.cum += elapsed_rel;
        self.previous = now;

        let timing = Timing {
            elapsed_cum: self.cum,
            elapsed_rel,
        };
        let trace = GasSpan {
            context,
            point,
            consumption,
            timing,
        };
        self.spans.push_back(trace)
    }
}

#[derive(Debug, Serialize)]
pub struct GasSpan {
    /// Context annotates the trace with the source context.
    #[serde(flatten)]
    pub context: Context,
    /// Point represents the location from where the trace was emitted.
    #[serde(flatten)]
    pub point: Point,
    /// The consumption at the moment of trace.
    #[serde(flatten)]
    pub consumption: Consumption,
    /// Timing information.
    #[serde(flatten)]
    pub timing: Timing,
}

#[derive(Debug, Serialize, Default)]
pub struct Consumption {
    /// Wasmtime fuel consumed reports how much fuel has been consumed at this point.
    /// May be optional if the point had no access to this information, or if non applicable.
    pub fuel_consumed: Option<u64>,
    /// Gas consumed reports how much gas has been consumed at this point.
    /// May be optional if the point had no access to this information, or if non applicable.
    pub gas_consumed: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct Timing {
    /// Total time elapsed since the GasTracer was created.
    #[serde(
        serialize_with = "ser::serialize_duration_as_nanos",
        rename = "elapsed_cum_ns"
    )]
    pub elapsed_cum: Duration,
    /// Relative time elapsed since the previous trace was recorded.
    #[serde(
        serialize_with = "ser::serialize_duration_as_nanos",
        rename = "elapsed_rel_ns"
    )]
    pub elapsed_rel: Duration,
}

#[derive(Debug, Serialize, Default)]
pub struct Context {
    #[serde(serialize_with = "ser::serialize_cid")]
    pub code_cid: Cid,
    pub method_num: MethodNum,
}

#[derive(Debug, Serialize)]
pub struct Point {
    pub event: Event,
    pub label: String,
}

#[derive(Debug, Serialize)]
pub enum Event {
    Started,
    EnterCall,
    PreSyscall,
    PostSyscall,
    PreExtern,
    PostExtern,
    ExitCall,
    Finished,
}

#[test]
fn test_tracer() {
    let mut trace = GasTrace::start();
    trace.record(
        Context {
            code_cid: Default::default(),
            method_num: 0,
        },
        Point {
            event: Event::Started,
            label: "".to_string(),
        },
        Consumption {
            fuel_consumed: None,
            gas_consumed: None,
        },
    );

    std::thread::sleep(Duration::from_millis(1000));
    trace.record(
        Context {
            code_cid: Default::default(),
            method_num: 0,
        },
        Point {
            event: Event::Started,
            label: "".to_string(),
        },
        Consumption {
            fuel_consumed: None,
            gas_consumed: None,
        },
    );
    println!("{:?}", trace);

    let str = serde_json::to_string(&trace).unwrap();
    println!("{}", str);
}

mod ser {
    use std::time::Duration;

    use cid::Cid;
    use serde::{Serialize, Serializer};

    pub fn serialize_cid<S>(c: &Cid, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        c.to_string().serialize(serializer)
    }

    pub fn serialize_duration_as_nanos<S>(d: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        d.as_nanos().serialize(serializer)
    }
}