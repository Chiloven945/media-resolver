use media_resolver_core::{ResourceBundle, ResourceItem};
use serde::Serialize;

use crate::runner::{PublicFailure, ResolveResult, ResultState};

#[derive(Serialize)]
pub struct JsonOutput<'a> {
    pub schema_version: u32,
    pub results: Vec<JsonResult<'a>>,
}

#[derive(Serialize)]
pub struct JsonResult<'a> {
    pub input: &'a str,
    pub state: ResultState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_key: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<&'a [ResourceItem]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<&'a PublicFailure>,
}

pub fn trim_variants(results: &mut [ResolveResult], all_variants: bool) {
    if all_variants {
        return;
    }

    for result in results {
        if let Some(bundle) = result.result.as_mut() {
            for resource in &mut bundle.resources {
                if let Some(preferred) = resource
                    .variants
                    .iter()
                    .find(|variant| variant.url == resource.preferred_url)
                    .cloned()
                {
                    resource.variants = vec![preferred];
                } else {
                    resource.variants.clear();
                }
            }
        }
    }
}

pub fn print_json(results: &[ResolveResult], pretty: bool) -> Result<(), serde_json::Error> {
    let output = JsonOutput {
        schema_version: 1,
        results: results.iter().map(json_result).collect(),
    };
    if pretty {
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("{}", serde_json::to_string(&output)?);
    }
    Ok(())
}

fn json_result(result: &ResolveResult) -> JsonResult<'_> {
    JsonResult {
        input: &result.input,
        state: result.state,
        source_key: result
            .result
            .as_ref()
            .map(|bundle| bundle.source_key.as_str()),
        resources: result
            .result
            .as_ref()
            .map(|bundle| bundle.resources.as_slice()),
        error: result.error.as_ref(),
    }
}

pub fn print_human(results: &[ResolveResult], color: bool) {
    for result in results {
        match result.state {
            ResultState::Ready => {
                if let Some(bundle) = result.result.as_ref() {
                    print_bundle(bundle);
                }
            }
            ResultState::Failed => {
                if let Some(error) = result.error.as_ref() {
                    if color {
                        eprintln!(
                            "\x1b[31m{}: {} ({})\x1b[0m",
                            result.input, error.message, error.code
                        );
                    } else {
                        eprintln!("{}: {} ({})", result.input, error.message, error.code);
                    }
                }
            }
        }
    }
}

fn print_bundle(bundle: &ResourceBundle) {
    for resource in &bundle.resources {
        println!("{}", resource.preferred_url);
        (&resource.variants)
            .into_iter()
            .filter(|v| v.url != resource.preferred_url)
            .for_each(|variant| println!("{}", variant.url));
    }
}
