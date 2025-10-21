use chrono::{DateTime, Utc};
use poem_openapi::Object;
use serde::Serialize;
use sqlx::{PgPool, Pool, Postgres};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::Event;
use tracing::field::{Field, Visit};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::Layer;
use tracing_subscriber::{Registry, prelude::*};
use uuid::Uuid;

// The type of data we send over the channel
#[derive(Debug)]
pub struct LogMessage {
    span_id: Uuid,
    tool_use: bool,
    endpoint: String,
    message: String,
}

// The async worker context
pub struct AsyncDbWorker {
    // The receiver half of the channel, owned by the worker task
    pub rx: mpsc::Receiver<LogMessage>,
    pub db_client: Pool<Postgres>,
}

// The synchronous Layer that feeds the worker
pub struct PSqlLayer {
    // The sender half of the channel, wrapped in a thread-safe container
    pub tx: Arc<Mutex<mpsc::Sender<LogMessage>>>,
}

impl<S: tracing::Subscriber> Layer<S> for PSqlLayer {
    fn on_event(&self, event: &Event, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        let mut visitor = PsqlVisitor::new();

        // Tell the event to record all its fields into our visitor
        event.record(&mut visitor);

        // 1. Extract data synchronously
        let log_data = LogMessage {
            span_id: Uuid::parse_str(
                &visitor
                    .span_id
                    .unwrap_or_else(|| "span needs to exist".to_string()),
            )
            .unwrap_or_else(|_e| Uuid::new_v4()),
            tool_use: visitor.tool_use.unwrap_or_else(|| false),
            endpoint: visitor
                .endpoint
                .unwrap_or_else(|| "no endpoint provided".to_string()),
            message: visitor.message.unwrap_or_else(|| "no message".to_string()),
        };

        // 2. Send data to the background worker (this is non-blocking)
        if let Ok(sender) = self.tx.lock() {
            let _ = sender.try_send(log_data);
        }
    }
    // ... other synchronous trait methods
}

pub async fn run_async_worker(mut worker: AsyncDbWorker) -> Result<(), sqlx::Error> {
    while let Some(log_message) = worker.rx.recv().await {
        let utc: DateTime<Utc> = Utc::now();
        println!("{}-{:?}", utc, log_message); //log to stdout as well
        let _ = sqlx::query!(
            r#"
            INSERT INTO traces (span_id, tool_use, endpoint, message, timestamp)
            VALUES ($1, $2, $3, $4, NOW())
            "#,
            &log_message.span_id,
            &log_message.tool_use,
            &log_message.endpoint,
            &log_message.message
        )
        .execute(&worker.db_client)
        .await?;
    }
    Ok(())
}

struct PsqlVisitor {
    /// The main log message, usually found under the "message" field.
    message: Option<String>,
    /// Stores the captured log level.
    tool_use: Option<bool>,
    endpoint: Option<String>,
    span_id: Option<String>,
    /// Stores any custom fields found.
    custom_fields: Vec<(String, String)>,
}

impl PsqlVisitor {
    fn new() -> Self {
        PsqlVisitor {
            message: None,
            span_id: None,
            tool_use: None,
            endpoint: None,
            custom_fields: Vec::new(),
        }
    }
}

/// Implement the Visit trait to define how different field types are extracted.
impl Visit for PsqlVisitor {
    /// Handler for string fields, primarily used to capture the main "message".
    fn record_str(&mut self, field: &Field, value: &str) {
        match field.name() {
            "message" => {
                self.message = Some(value.to_string());
            }
            "span_id" => {
                self.span_id = Some(value.to_string());
            }
            "endpoint" => {
                self.endpoint = Some(value.to_string());
            }
            _ => {
                self.custom_fields
                    .push((field.name().to_string(), value.to_string()));
            }
        }
    }

    /// Handler for debug-printable fields (the default fallback).
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        // We use this for all types other than string/integers we specifically care about.
        if field.name() == "message" {
            // Even if the message isn't a string literal, try to capture it.
            self.message = Some(format!("{:?}", value));
        } else {
            // Capture custom fields as debug strings
            self.custom_fields
                .push((field.name().to_string(), format!("{:?}", value)));
        }
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        match field.name() {
            "tool_use" => {
                self.tool_use = Some(value);
            }
            _ => {
                self.custom_fields
                    .push((field.name().to_string(), value.to_string()));
            }
        }
    }
}

#[derive(Serialize)]
pub struct SpanLength {
    span_id: Uuid,
    diff_in_seconds: f64,
}

fn hist_bin_num(data_size: usize) -> i32 {
    let bin = (data_size as f32).ln().ceil() as i32;
    let global_min = 5;
    let global_max = 20;
    if bin < global_min {
        global_min
    } else if bin > global_max {
        global_max
    } else {
        bin
    }
}

#[derive(Serialize, Debug, Object)]
pub struct HistogramIncrement {
    index: i32,
    range: String,
    count: usize,
}
//spans are sorted by diff_in_seconds ascending
fn extract_histogram(spans: &[SpanLength]) -> Vec<HistogramIncrement> {
    if spans.is_empty() {
        return vec![];
    }
    let min_value_in_data = spans.first().unwrap().diff_in_seconds;
    let max_value_in_data = spans.last().unwrap().diff_in_seconds;
    let num_bins = hist_bin_num(spans.len());
    let min_value_in_hist = min_value_in_data.floor() - 1.0; //ensure that it is below on exact "integers"
    let max_value_in_hist = max_value_in_data.ceil() + 1.0; //ensure that it is above on exact "integers"
    let increment = (max_value_in_hist - min_value_in_hist) / (num_bins as f64);
    let ranges: Vec<HistogramIncrement> = (0..num_bins)
        .map(|i| {
            let left_bin = min_value_in_hist + (i as f64) * increment;
            (i, left_bin, left_bin + increment)
        })
        .map(|(i, left, right)| HistogramIncrement {
            index: i,
            range: format!("{:.2}-{:.2}", left, right),
            count: spans //o(n^2), but super clear that this is correct
                .iter()
                .filter(|v| left <= v.diff_in_seconds && v.diff_in_seconds < right)
                .count(),
        })
        .collect();
    ranges
}

pub async fn get_histogram(
    pool: &PgPool,
    endpoint: &str,
) -> Result<Vec<HistogramIncrement>, sqlx::Error> {
    let spans = sqlx::query_as!(
        SpanLength,
        r#"
        SELECT diff_in_seconds as "diff_in_seconds!", span_id FROM
        (
            SELECT COALESCE(EXTRACT(EPOCH FROM (MAX(timestamp) - MIN(timestamp))), 0)::double precision as diff_in_seconds,
            span_id from
            traces
            where timestamp> date_subtract(NOW(), '7 day'::interval)
            and endpoint=$1
            group by span_id
            order by diff_in_seconds asc
        )
        "#,
        endpoint
    )
    .fetch_all(pool)
    .await?;
    let histogram = extract_histogram(&spans);

    Ok(histogram)
}

#[derive(Serialize, Object)]
pub struct SpanToolUse {
    cnt_spns_with_tools: i64,
    cnt_spns_without_tools: i64,
    date: chrono::DateTime<chrono::Utc>,
}
pub async fn get_tool_use(pool: &PgPool, endpoint: &str) -> Result<Vec<SpanToolUse>, sqlx::Error> {
    let spans = sqlx::query_as!(
        SpanToolUse,
        r#"
            SELECT
            SUM(CASE WHEN used_tools then 1 else 0 end) as "cnt_spns_with_tools!",
            SUM(CASE WHEN not used_tools then 1 else 0 end) as "cnt_spns_without_tools!",
            date as "date!" FROM
            (
                SELECT CASE WHEN
                    MAX(CASE WHEN tool_use is true then 1 else 0 END)=1
                    then true else false END as used_tools,
                    date_trunc('day', timestamp) as date,
                span_id from
                traces
                where timestamp> date_subtract(NOW(), '7 day'::interval)
                and endpoint=$1
                group by span_id, date_trunc('day', timestamp)
            )
            group by date
            order by date asc
            "#,
        endpoint
    ) //-- where timestamp> NOW()::DATE-EXTRACT(DOW FROM NOW())::INTEGER-7
    .fetch_all(pool)
    .await?;
    Ok(spans)
}

pub fn create_logging(db: &PgPool) -> JoinHandle<Result<(), sqlx::Error>> {
    let (tx, rx) = mpsc::channel(100);

    // Spawn the worker task onto the tokio runtime
    let worker_handle = tokio::spawn(run_async_worker(AsyncDbWorker {
        rx,
        db_client: db.clone(),
    }));

    let layer = PSqlLayer {
        tx: Arc::new(Mutex::new(tx)),
    };

    // Optional: Add an EnvFilter layer for runtime filtering
    let filter_layer = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    let subscriber = Registry::default()
        .with(filter_layer) // Handles RUST_LOG environment variable filtering
        .with(layer); // Your custom processing layer

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global tracing subscriber");

    worker_handle
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::dbtracing::HistogramIncrement;

    use super::SpanLength;
    use super::extract_histogram;
    use super::hist_bin_num;

    #[test]
    fn it_gets_correct_hist_bin_when_small() {
        let result = hist_bin_num(4);
        assert!(result == 5);
    }
    #[test]
    fn it_gets_correct_hist_bin_when_large() {
        let result = hist_bin_num(10000000000);
        assert!(result == 20);
    }
    #[test]
    fn it_gets_correct_hist_bin_when_middle() {
        let result = hist_bin_num(1000);
        assert!(result == 7);
    }
    #[test]
    fn it_returns_correctly_composes_histogram() {
        let spans = vec![
            SpanLength {
                span_id: Uuid::new_v4(),
                diff_in_seconds: 3.5,
            },
            SpanLength {
                span_id: Uuid::new_v4(),
                diff_in_seconds: 3.5,
            },
            SpanLength {
                span_id: Uuid::new_v4(),
                diff_in_seconds: 4.5,
            },
            SpanLength {
                span_id: Uuid::new_v4(),
                diff_in_seconds: 4.5,
            },
            SpanLength {
                span_id: Uuid::new_v4(),
                diff_in_seconds: 5.5,
            },
            SpanLength {
                span_id: Uuid::new_v4(),
                diff_in_seconds: 5.5,
            },
            SpanLength {
                span_id: Uuid::new_v4(),
                diff_in_seconds: 5.5,
            },
        ];
        let result = extract_histogram(&spans);
        let expectation = vec![
            HistogramIncrement {
                index: 0,
                range: "2.00-3.00".to_string(),
                count: 0,
            },
            HistogramIncrement {
                index: 1,
                range: "3.00-4.00".to_string(),
                count: 2,
            },
            HistogramIncrement {
                index: 2,
                range: "4.00-5.00".to_string(),
                count: 2,
            },
            HistogramIncrement {
                index: 3,
                range: "5.00-6.00".to_string(),
                count: 3,
            },
            HistogramIncrement {
                index: 4,
                range: "6.00-7.00".to_string(),
                count: 0,
            },
        ];
        for (res, exp) in result.iter().zip(expectation) {
            assert!(res.count == exp.count);
            assert!(res.range == exp.range);
        }
    }
    #[test]
    fn it_returns_correctly_histogram_with_zero_elements() {
        let spans = vec![];
        let result = extract_histogram(&spans);
        assert!(result.is_empty());
    }
    #[test]
    fn it_returns_correctly_composes_histogram_with_one_element() {
        let spans = vec![SpanLength {
            span_id: Uuid::new_v4(),
            diff_in_seconds: 2.5,
        }];
        let result = extract_histogram(&spans);
        let expectation = vec![
            HistogramIncrement {
                index: 0,
                range: "1.00-1.60".to_string(),
                count: 0,
            },
            HistogramIncrement {
                index: 1,
                range: "1.60-2.20".to_string(),
                count: 0,
            },
            HistogramIncrement {
                index: 2,
                range: "2.20-2.80".to_string(),
                count: 1,
            },
            HistogramIncrement {
                index: 3,
                range: "2.80-3.40".to_string(),
                count: 0,
            },
            HistogramIncrement {
                index: 4,
                range: "3.40-4.00".to_string(),
                count: 0,
            },
        ];
        for (res, exp) in result.iter().zip(expectation) {
            assert!(res.count == exp.count);
            assert!(res.range == exp.range);
        }
    }
    #[test]
    fn it_returns_correctly_composes_histogram_with_one_element_at_integer() {
        let spans = vec![SpanLength {
            span_id: Uuid::new_v4(),
            diff_in_seconds: 2.0,
        }];
        let result = extract_histogram(&spans);
        let expectation = vec![
            HistogramIncrement {
                index: 0,
                range: "1.00-1.40".to_string(),
                count: 0,
            },
            HistogramIncrement {
                index: 1,
                range: "1.40-1.80".to_string(),
                count: 0,
            },
            HistogramIncrement {
                index: 2,
                range: "1.80-2.20".to_string(),
                count: 1,
            },
            HistogramIncrement {
                index: 3,
                range: "2.20-2.60".to_string(),
                count: 0,
            },
            HistogramIncrement {
                index: 4,
                range: "2.60-3.00".to_string(),
                count: 0,
            },
        ];
        for (res, exp) in result.iter().zip(expectation) {
            println!("result {:?}", res);
            assert!(res.count == exp.count);
            assert!(res.range == exp.range);
        }
    }
}
