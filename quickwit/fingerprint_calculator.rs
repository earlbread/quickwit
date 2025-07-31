use std::num::NonZeroUsize;

use quickwit_config::{
    indexing_pipeline_params_fingerprint, IndexConfig, KafkaSourceParams, SourceConfig,
    SourceInputFormat, SourceParams, INGEST_API_SOURCE_ID,
};

fn main() {
    // Create the index config using the test helper
    let index_config = IndexConfig::for_test("test-index", "ram:///indexes/test-index");

    // Create the three source configs as used in the test
    
    // 1. Ingest API source (default)
    let ingest_api_source = SourceConfig::ingest_api_default();
    
    // 2. Source-1 with void params
    let source_config_1 = SourceConfig {
        source_id: "test-indexing-service--source-1".to_string(),
        num_pipelines: NonZeroUsize::MIN,
        enabled: true,
        source_params: SourceParams::void(),
        transform_config: None,
        input_format: SourceInputFormat::Json,
    };
    
    // 3. Source-2 with kafka params (exact same as in the test)
    let kafka_params = KafkaSourceParams {
        topic: "my-topic".to_string(),
        client_log_level: None,
        client_params: serde_json::Value::Null,
        enable_backfill_mode: false,
    };
    let source_config_2 = SourceConfig {
        source_id: "test-indexing-service--source-2".to_string(),
        num_pipelines: NonZeroUsize::new(2).unwrap(),
        enabled: true,
        source_params: SourceParams::Kafka(kafka_params),
        transform_config: None,
        input_format: SourceInputFormat::Json,
    };

    // Calculate fingerprints
    let fingerprint_ingest_api = indexing_pipeline_params_fingerprint(&index_config, &ingest_api_source);
    let fingerprint_source_1 = indexing_pipeline_params_fingerprint(&index_config, &source_config_1);
    let fingerprint_source_2 = indexing_pipeline_params_fingerprint(&index_config, &source_config_2);

    println!("New fingerprint values:");
    println!("PARAMS_FINGERPRINT_INGEST_API: u64 = {}", fingerprint_ingest_api);
    println!("PARAMS_FINGERPRINT_SOURCE_1: u64 = {}", fingerprint_source_1);
    println!("PARAMS_FINGERPRINT_SOURCE_2: u64 = {}", fingerprint_source_2);
    
    println!("\nConstants to update in the test:");
    println!("const PARAMS_FINGERPRINT_INGEST_API: u64 = {};", fingerprint_ingest_api);
    println!("const PARAMS_FINGERPRINT_SOURCE_1: u64 = {};", fingerprint_source_1);
    println!("const PARAMS_FINGERPRINT_SOURCE_2: u64 = {};", fingerprint_source_2);
}