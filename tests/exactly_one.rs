//! `exactly_one(...)` is authoring sugar: it must lower to the same
//! Or-of-And-of-Nots expansion encoders previously wrote by hand, so the
//! compiled program, evaluation, and traces are indistinguishable from the
//! manual form.

use axiom_rules_engine::api::{
    ExecutionMode, ExecutionQuery, ExecutionRequest, OutputValue, execute_request,
};
use axiom_rules_engine::spec::{
    DatasetSpec, InputRecordSpec, IntervalSpec, JudgmentOutcomeSpec, PeriodKindSpec, PeriodSpec,
    ScalarValueSpec,
};

const SUGARED_RULESPEC: &str = r#"
format: rulespec/v1
rules:
  - name: filing_status_is_valid
    kind: derived
    entity: TaxUnit
    dtype: Judgment
    versions:
      - effective_from: 2026-01-01
        formula: exactly_one(status_single, status_married_separate, status_joint, status_head_of_household)
"#;

const EXPANDED_RULESPEC: &str = r#"
format: rulespec/v1
rules:
  - name: filing_status_is_valid
    kind: derived
    entity: TaxUnit
    dtype: Judgment
    versions:
      - effective_from: 2026-01-01
        formula: |-
          (
            status_single
            and not status_married_separate
            and not status_joint
            and not status_head_of_household
          )
          or (
            not status_single
            and status_married_separate
            and not status_joint
            and not status_head_of_household
          )
          or (
            not status_single
            and not status_married_separate
            and status_joint
            and not status_head_of_household
          )
          or (
            not status_single
            and not status_married_separate
            and not status_joint
            and status_head_of_household
          )
"#;

fn simple_period() -> PeriodSpec {
    PeriodSpec {
        kind: PeriodKindSpec::Month,
        start: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid date"),
        end: chrono::NaiveDate::from_ymd_opt(2026, 1, 31).expect("valid date"),
    }
}

fn judgment_output(output: &OutputValue) -> JudgmentOutcomeSpec {
    match output {
        OutputValue::Judgment { outcome, .. } => *outcome,
        other => panic!("expected judgment output, got {other:?}"),
    }
}

fn status_dataset(period: &PeriodSpec, statuses: [bool; 4]) -> DatasetSpec {
    let names = [
        "status_single",
        "status_married_separate",
        "status_joint",
        "status_head_of_household",
    ];
    DatasetSpec {
        inputs: names
            .into_iter()
            .zip(statuses)
            .map(|(name, value)| InputRecordSpec {
                name: name.to_string(),
                entity: "TaxUnit".to_string(),
                entity_id: "tax-unit-1".to_string(),
                interval: IntervalSpec {
                    start: period.start,
                    end: period.end,
                },
                value: ScalarValueSpec::Bool { value },
            })
            .collect(),
        relations: vec![],
    }
}

fn run(rulespec: &str, statuses: [bool; 4]) -> JudgmentOutcomeSpec {
    let period = simple_period();
    let program =
        axiom_rules_engine::rulespec::lower_rulespec_str(rulespec).expect("RuleSpec lowers");
    let response = execute_request(ExecutionRequest {
        mode: ExecutionMode::Explain,
        program,
        dataset: status_dataset(&period, statuses),
        queries: vec![ExecutionQuery {
            assessment_date: None,
            entity_id: "tax-unit-1".to_string(),
            period,
            outputs: vec!["filing_status_is_valid".to_string()],
        }],
    })
    .expect("request executes");
    judgment_output(
        response.results[0]
            .outputs
            .get("filing_status_is_valid")
            .expect("judgment output"),
    )
}

#[test]
fn exactly_one_lowers_to_a_flat_or_of_and_of_nots() {
    let program = axiom_rules_engine::rulespec::lower_rulespec_str(SUGARED_RULESPEC)
        .expect("sugared RuleSpec lowers");
    let program = serde_json::to_value(&program).expect("program serialises");
    let expr = program
        .pointer("/derived/0/expr")
        .expect("judgment expression is serialised");

    let holds = |name: &str| {
        serde_json::json!({
            "kind": "comparison",
            "left": {"kind": "input", "name": name},
            "op": "eq",
            "right": {"kind": "literal", "value": {"kind": "bool", "value": true}},
        })
    };
    let not_holds = |name: &str| serde_json::json!({"kind": "not", "item": holds(name)});
    let names = [
        "status_single",
        "status_married_separate",
        "status_joint",
        "status_head_of_household",
    ];
    let expected = serde_json::json!({
        "kind": "or",
        "items": names
            .iter()
            .enumerate()
            .map(|(branch, _)| {
                serde_json::json!({
                    "kind": "and",
                    "items": names
                        .iter()
                        .enumerate()
                        .map(|(position, name)| if position == branch {
                            holds(name)
                        } else {
                            not_holds(name)
                        })
                        .collect::<Vec<_>>(),
                })
            })
            .collect::<Vec<_>>(),
    });
    assert_eq!(
        expr, &expected,
        "exactly_one must lower to one flat Or over n And-of-Nots branches",
    );
}

#[test]
fn exactly_one_matches_the_expansion_on_every_input_combination() {
    for bits in 0..16u8 {
        let statuses = [bits & 1 != 0, bits & 2 != 0, bits & 4 != 0, bits & 8 != 0];
        let sugared = run(SUGARED_RULESPEC, statuses);
        let expanded = run(EXPANDED_RULESPEC, statuses);
        assert_eq!(sugared, expanded, "divergence at inputs {statuses:?}");
        let expected = if statuses.iter().filter(|held| **held).count() == 1 {
            JudgmentOutcomeSpec::Holds
        } else {
            JudgmentOutcomeSpec::NotHolds
        };
        assert_eq!(sugared, expected, "wrong outcome at inputs {statuses:?}");
    }
}

#[test]
fn exactly_one_accepts_arbitrary_judgment_arguments() {
    let rulespec = r#"
format: rulespec/v1
rules:
  - name: single_path_applies
    kind: derived
    entity: TaxUnit
    dtype: Judgment
    versions:
      - effective_from: 2026-01-01
        formula: exactly_one(status_single, monthly_income > 1000, not status_joint)
"#;
    axiom_rules_engine::rulespec::lower_rulespec_str(rulespec)
        .expect("judgment-shaped arguments lower");
}

#[test]
fn exactly_one_rejects_fewer_than_two_arguments() {
    let rulespec = r#"
format: rulespec/v1
rules:
  - name: degenerate
    kind: derived
    entity: TaxUnit
    dtype: Judgment
    versions:
      - effective_from: 2026-01-01
        formula: exactly_one(status_single)
"#;
    let error = axiom_rules_engine::rulespec::lower_rulespec_str(rulespec)
        .expect_err("single-argument exactly_one must be rejected")
        .to_string();
    assert!(
        error.contains("exactly_one takes at least 2 arguments"),
        "error must name the arity contract, got: {error}",
    );
}

#[test]
fn unknown_judgment_functions_are_named_in_the_error() {
    let rulespec = r#"
format: rulespec/v1
rules:
  - name: bad_call
    kind: derived
    entity: TaxUnit
    dtype: Judgment
    versions:
      - effective_from: 2026-01-01
        formula: only_one(status_single, status_joint)
"#;
    let error = axiom_rules_engine::rulespec::lower_rulespec_str(rulespec)
        .expect_err("unknown judgment function must be rejected")
        .to_string();
    assert!(
        error.contains("unknown function `only_one` in judgment position"),
        "error must name the unknown function, got: {error}",
    );
}
