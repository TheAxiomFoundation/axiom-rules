use std::collections::{BTreeMap, BTreeSet};

use num_bigint::BigInt;
use serde::{Deserialize, Serialize};

use super::types::*;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AggregationPlan {
    pub schema: String,
    pub id: String,
    pub decimal_precision: u32,
    pub constitution: AggregationConstitution,
    pub membership_relations: Vec<AggregationMembershipRelation>,
    pub partner_relation: String,
    pub child_relation: String,
    #[serde(default)]
    pub scalar_aggregations: Vec<ScalarAggregation>,
    #[serde(default)]
    pub child_counts: Vec<ChildCount>,
    #[serde(default)]
    pub child_minima: Vec<ChildMinimum>,
    #[serde(default)]
    pub broadcasts: Vec<ChildBroadcast>,
    #[serde(default)]
    pub family_reductions: Vec<FamilyReduction>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AggregationConstitution {
    pub id: String,
    pub entity_type: String,
    pub roster_relation: String,
    pub unit_constituent_relation: String,
    pub participating_member_relation: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AggregationMembershipRelation {
    pub relation: String,
    #[serde(default)]
    pub symmetric: bool,
    pub citation: AggregationCitation,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AggregationCitation {
    pub provision: String,
    pub authority: String,
}

impl From<&AggregationCitation> for Citation {
    fn from(value: &AggregationCitation) -> Self {
        Citation::new(value.provision.clone(), value.authority.clone())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AggregateSelector {
    Adults,
    AllMembers,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScalarAggregation {
    pub output: String,
    pub input: String,
    pub selector: AggregateSelector,
    pub citation: AggregationCitation,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChildCount {
    pub output: String,
    #[serde(default)]
    pub minimum_age: Option<i64>,
    #[serde(default)]
    pub maximum_age: Option<i64>,
    pub citation: AggregationCitation,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChildMinimum {
    pub output: String,
    pub input: String,
    pub empty_value: i64,
    pub citation: AggregationCitation,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChildBroadcast {
    pub output: String,
    pub family_input: String,
    pub citation: AggregationCitation,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FamilyReduction {
    pub output: String,
    pub gross_output: String,
    pub operation: FamilyReductionOperation,
    pub child_value: String,
    pub family_once_value: String,
    #[serde(default)]
    pub minimum_age: Option<i64>,
    #[serde(default)]
    pub maximum_age: Option<i64>,
    pub citation: AggregationCitation,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FamilyReductionOperation {
    /// Sum per-child gross credits, require every child-carried family value
    /// to agree, and subtract that value exactly once from the family total.
    SumChildrenThenSubtractFamilyOnce,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AggregationRequest {
    pub scope: String,
    pub segment: String,
    pub primary_person: String,
    pub persons: Vec<AggregationPerson>,
    #[serde(default)]
    pub relations: BTreeMap<String, Vec<[String; 2]>>,
    #[serde(default)]
    pub family_scalars: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AggregationPerson {
    pub id: String,
    #[serde(default)]
    pub age_years: Option<i64>,
    #[serde(default)]
    pub scalars: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct AggregationResult {
    pub schema: String,
    pub plan: String,
    pub families: Vec<AggregatedFamily>,
    pub trace_roots: Vec<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct AggregatedFamily {
    pub id: String,
    pub members: Vec<String>,
    pub partner_present: bool,
    pub scalars: BTreeMap<String, String>,
    pub counts: BTreeMap<String, i64>,
    pub children: Vec<AggregatedChild>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct AggregatedChild {
    pub person: String,
    pub family: String,
    pub age_years: i64,
    pub age_bands: BTreeMap<String, bool>,
    pub scalars: BTreeMap<String, String>,
}

pub fn parse_aggregation_plan(source: &str) -> Result<AggregationPlan, UnitDerivationError> {
    let plan: AggregationPlan = serde_yaml::from_str(source).map_err(|error| {
        UnitDerivationError::InvalidPlan(format!("invalid aggregation plan YAML: {error}"))
    })?;
    validate_aggregation_plan(&plan)?;
    Ok(plan)
}

fn validate_aggregation_plan(plan: &AggregationPlan) -> Result<(), UnitDerivationError> {
    if plan.schema != super::EXPERIMENTAL_AGGREGATION_PLAN_SCHEMA {
        return Err(UnitDerivationError::InvalidPlan(format!(
            "aggregation plan schema `{}` is unsupported",
            plan.schema
        )));
    }
    if plan.id.is_empty()
        || plan.constitution.id.is_empty()
        || plan.constitution.entity_type.is_empty()
        || plan.constitution.roster_relation.is_empty()
    {
        return Err(UnitDerivationError::InvalidPlan(
            "aggregation plan and constitution identities must be non-empty".to_string(),
        ));
    }
    if plan.decimal_precision == 0 {
        return Err(UnitDerivationError::InvalidPlan(
            "aggregation decimal_precision must be positive".to_string(),
        ));
    }
    let mut relations = BTreeSet::new();
    for relation in &plan.membership_relations {
        if relation.relation.is_empty() || !relations.insert(relation.relation.clone()) {
            return Err(UnitDerivationError::DuplicateNamespace {
                namespace: "aggregation membership relation",
                id: relation.relation.clone(),
            });
        }
        if relation.citation.provision.is_empty() || relation.citation.authority.is_empty() {
            return Err(UnitDerivationError::InvalidPlan(format!(
                "membership relation `{}` needs a complete citation",
                relation.relation
            )));
        }
    }
    for required in [&plan.partner_relation, &plan.child_relation] {
        if !relations.contains(required) {
            return Err(UnitDerivationError::UnknownReference {
                from: plan.id.clone(),
                reference: required.clone(),
            });
        }
    }

    let mut outputs = BTreeSet::new();
    for aggregation in &plan.scalar_aggregations {
        validate_citation(&aggregation.citation, &aggregation.output)?;
        insert_output(&mut outputs, &aggregation.output)?;
    }
    for count in &plan.child_counts {
        validate_citation(&count.citation, &count.output)?;
        validate_age_range(count.minimum_age, count.maximum_age, &count.output)?;
        insert_output(&mut outputs, &count.output)?;
    }
    for minimum in &plan.child_minima {
        validate_citation(&minimum.citation, &minimum.output)?;
        insert_output(&mut outputs, &minimum.output)?;
    }
    for broadcast in &plan.broadcasts {
        validate_citation(&broadcast.citation, &broadcast.output)?;
        insert_output(&mut outputs, &broadcast.output)?;
    }
    for reduction in &plan.family_reductions {
        validate_citation(&reduction.citation, &reduction.output)?;
        validate_age_range(
            reduction.minimum_age,
            reduction.maximum_age,
            &reduction.output,
        )?;
        insert_output(&mut outputs, &reduction.output)?;
        insert_output(&mut outputs, &reduction.gross_output)?;
        if plan.scalar_aggregations.iter().any(|aggregation| {
            aggregation.input == reduction.child_value
                || aggregation.input == reduction.family_once_value
                || aggregation.output == reduction.output
                || aggregation.output == reduction.gross_output
        }) {
            return Err(UnitDerivationError::InvalidPlan(format!(
                "family-once reduction `{}` reserves its child, abatement, and output fields from ordinary scalar aggregation",
                reduction.output
            )));
        }
    }
    Ok(())
}

fn validate_citation(
    citation: &AggregationCitation,
    operation: &str,
) -> Result<(), UnitDerivationError> {
    if citation.provision.is_empty() || citation.authority.is_empty() {
        return Err(UnitDerivationError::InvalidPlan(format!(
            "aggregation operation `{operation}` needs a complete citation"
        )));
    }
    Ok(())
}

fn insert_output(outputs: &mut BTreeSet<String>, output: &str) -> Result<(), UnitDerivationError> {
    if output.is_empty() || !outputs.insert(output.to_string()) {
        return Err(UnitDerivationError::DuplicateNamespace {
            namespace: "aggregation output",
            id: output.to_string(),
        });
    }
    Ok(())
}

fn validate_age_range(
    minimum: Option<i64>,
    maximum: Option<i64>,
    output: &str,
) -> Result<(), UnitDerivationError> {
    if minimum
        .zip(maximum)
        .is_some_and(|(minimum, maximum)| minimum > maximum)
    {
        return Err(UnitDerivationError::InvalidPlan(format!(
            "age range for `{output}` is reversed"
        )));
    }
    Ok(())
}

pub fn execute_aggregation_plan(
    plan: &AggregationPlan,
    request: &AggregationRequest,
    config: &UnitDerivationConfig,
) -> Result<AggregationResult, UnitDerivationError> {
    validate_aggregation_plan(plan)?;
    let people = request
        .persons
        .iter()
        .map(|person| (person.id.clone(), person))
        .collect::<BTreeMap<_, _>>();
    if people.len() != request.persons.len() {
        return Err(UnitDerivationError::DuplicateNamespace {
            namespace: "aggregation person",
            id: request
                .persons
                .iter()
                .find(|person| {
                    request
                        .persons
                        .iter()
                        .filter(|other| other.id == person.id)
                        .count()
                        > 1
                })
                .map(|person| person.id.clone())
                .unwrap_or_default(),
        });
    }
    if !people.contains_key(&request.primary_person) {
        return Err(UnitDerivationError::InvalidPlan(format!(
            "primary person `{}` is outside the roster",
            request.primary_person
        )));
    }
    if request.scope.is_empty() || request.segment.is_empty() || people.is_empty() {
        return Err(UnitDerivationError::InvalidPlan(
            "aggregation scope, segment, and roster must be non-empty".to_string(),
        ));
    }

    let relation_plans = plan
        .membership_relations
        .iter()
        .map(|relation| (relation.relation.as_str(), relation))
        .collect::<BTreeMap<_, _>>();
    for (name, tuples) in &request.relations {
        if !relation_plans.contains_key(name.as_str()) {
            return Err(UnitDerivationError::UnknownReference {
                from: "aggregation request relation".to_string(),
                reference: name.clone(),
            });
        }
        for tuple in tuples {
            if tuple[0] == tuple[1]
                || !people.contains_key(&tuple[0])
                || !people.contains_key(&tuple[1])
            {
                return Err(UnitDerivationError::InvalidPlan(format!(
                    "relation `{name}` contains invalid roster tuple {:?}",
                    tuple
                )));
            }
        }
    }

    let mut constitution = ConstitutionPlan {
        id: plan.constitution.id.clone(),
        entity_type: plan.constitution.entity_type.clone(),
        roster_relation: plan.constitution.roster_relation.clone(),
        relations: EmissionRelations {
            unit_constituent: plan.constitution.unit_constituent_relation.clone(),
            participating_member: plan.constitution.participating_member_relation.clone(),
        },
        derived_bools: Vec::new(),
        edges: Vec::new(),
        cuts: Vec::new(),
        attachments: Vec::new(),
        bars: Vec::new(),
        statuses: Vec::new(),
        base_chain_policy: None,
    };
    let person_ids = people.keys().cloned().collect::<Vec<_>>();
    for relation in &plan.membership_relations {
        for left_index in 0..person_ids.len() {
            for right_index in (left_index + 1)..person_ids.len() {
                let left = &person_ids[left_index];
                let right = &person_ids[right_index];
                let mut alternatives = vec![BoolExpr::fact(FactRef::Relation {
                    family: relation.relation.clone(),
                    tuple: vec![left.clone(), right.clone()],
                })];
                // Membership edges are undirected even when the evidence
                // relation (for example caregiver -> child) is directional.
                alternatives.push(BoolExpr::fact(FactRef::Relation {
                    family: relation.relation.clone(),
                    tuple: vec![right.clone(), left.clone()],
                }));
                constitution.edges.push(EdgeRule {
                    id: format!("{}:{left}:{right}", relation.relation),
                    kind: EdgeKind::Combination,
                    left: left.clone(),
                    right: right.clone(),
                    when: BoolExpr::Or(alternatives),
                    citation: (&relation.citation).into(),
                    defeaters: Vec::new(),
                });
            }
        }
    }

    let compiled = super::compile(constitution)?;
    let relation_families = plan
        .membership_relations
        .iter()
        .map(|relation| {
            let facts = request
                .relations
                .get(&relation.relation)
                .into_iter()
                .flatten()
                .map(|tuple| RelationFact {
                    tuple: tuple.to_vec(),
                    observation: ObservedBool {
                        value: true,
                        evidence: Evidence {
                            id: format!("{}:{}:{}", relation.relation, tuple[0], tuple[1]),
                            citation: (&relation.citation).into(),
                        },
                    },
                })
                .collect();
            RelationFamilyInput {
                name: relation.relation.clone(),
                scope: request.scope.clone(),
                completeness: Some(Evidence {
                    id: format!("complete:{}:{}", request.scope, relation.relation),
                    citation: (&relation.citation).into(),
                }),
                facts,
            }
        })
        .collect();
    let input = ConstitutionInput {
        roster: RosterInput {
            relation: plan.constitution.roster_relation.clone(),
            scope: request.scope.clone(),
            persons: person_ids,
            completeness: Some(Evidence {
                id: format!("complete-roster:{}", request.scope),
                citation: Citation::new(
                    "aggregation-plan-roster",
                    format!("{} explicit roster", plan.id),
                ),
            }),
        },
        segment: request.segment.clone(),
        segment_complete: true,
        relation_families,
        bool_facts: Vec::new(),
        supplied_entities: Vec::new(),
        integrity_constraints: Vec::new(),
    };
    let derivation = super::derive_units(&compiled, &input, config)?;
    if !derivation.indeterminate.is_empty() {
        return Err(UnitDerivationError::InvalidPlan(format!(
            "aggregation membership is indeterminate: {:?}",
            derivation.indeterminate
        )));
    }

    let child_ids = request
        .relations
        .get(&plan.child_relation)
        .into_iter()
        .flatten()
        .map(|tuple| tuple[1].clone())
        .collect::<BTreeSet<_>>();
    let partner_pairs = request
        .relations
        .get(&plan.partner_relation)
        .into_iter()
        .flatten()
        .map(|tuple| (tuple[0].clone(), tuple[1].clone()))
        .collect::<BTreeSet<_>>();

    let mut families = Vec::new();
    for unit in &derivation.units {
        let member_set = unit.members.iter().cloned().collect::<BTreeSet<_>>();
        let family_children = child_ids
            .intersection(&member_set)
            .cloned()
            .collect::<BTreeSet<_>>();
        let adults = member_set
            .difference(&family_children)
            .cloned()
            .collect::<BTreeSet<_>>();
        let partner_present = partner_pairs.iter().any(|(left, right)| {
            member_set.contains(left)
                && member_set.contains(right)
                && (left == &request.primary_person || right == &request.primary_person)
        });
        let mut scalars = BTreeMap::new();
        for aggregation in &plan.scalar_aggregations {
            let selected = match aggregation.selector {
                AggregateSelector::Adults => &adults,
                AggregateSelector::AllMembers => &member_set,
            };
            let mut total = ExactDecimal::zero();
            for person_id in selected {
                let raw = people[person_id]
                    .scalars
                    .get(&aggregation.input)
                    .ok_or_else(|| {
                        UnitDerivationError::InvalidPlan(format!(
                            "person `{person_id}` lacks scalar `{}`",
                            aggregation.input
                        ))
                    })?;
                total.add_assign(
                    parse_decimal(
                        raw,
                        &format!("person `{person_id}` scalar `{}`", aggregation.input),
                    )?,
                    plan.decimal_precision,
                );
            }
            scalars.insert(aggregation.output.clone(), decimal_text(total));
        }

        let mut counts = BTreeMap::new();
        for count in &plan.child_counts {
            let value = family_children
                .iter()
                .filter(|child| {
                    people[*child]
                        .age_years
                        .is_some_and(|age| age_in_range(age, count.minimum_age, count.maximum_age))
                })
                .count() as i64;
            counts.insert(count.output.clone(), value);
        }
        for minimum in &plan.child_minima {
            if minimum.input != "age_years" {
                return Err(UnitDerivationError::InvalidPlan(format!(
                    "child minimum `{}` uses unsupported input `{}`",
                    minimum.output, minimum.input
                )));
            }
            let value = family_children
                .iter()
                .filter_map(|child| people[child].age_years)
                .min()
                .unwrap_or(minimum.empty_value);
            counts.insert(minimum.output.clone(), value);
        }

        let mut children = Vec::new();
        for child_id in &family_children {
            let age = people[child_id].age_years.ok_or_else(|| {
                UnitDerivationError::InvalidPlan(format!("child `{child_id}` lacks age_years"))
            })?;
            let mut child_scalars = BTreeMap::new();
            let age_bands = plan
                .child_counts
                .iter()
                .map(|count| {
                    (
                        count.output.clone(),
                        age_in_range(age, count.minimum_age, count.maximum_age),
                    )
                })
                .collect();
            for broadcast in &plan.broadcasts {
                let raw = request
                    .family_scalars
                    .get(&broadcast.family_input)
                    .ok_or_else(|| {
                        UnitDerivationError::InvalidPlan(format!(
                            "family input `{}` required by child broadcast `{}` is missing",
                            broadcast.family_input, broadcast.output
                        ))
                    })?;
                child_scalars.insert(
                    broadcast.output.clone(),
                    decimal_text(parse_decimal(
                        raw,
                        &format!("family scalar `{}`", broadcast.family_input),
                    )?),
                );
            }
            children.push(AggregatedChild {
                person: child_id.clone(),
                family: unit.id.clone(),
                age_years: age,
                age_bands,
                scalars: child_scalars,
            });
        }
        children.sort_by(|left, right| left.person.cmp(&right.person));

        for reduction in &plan.family_reductions {
            let eligible = family_children
                .iter()
                .filter(|child| {
                    people[*child].age_years.is_some_and(|age| {
                        age_in_range(age, reduction.minimum_age, reduction.maximum_age)
                    })
                })
                .collect::<Vec<_>>();
            let mut child_total = ExactDecimal::zero();
            let mut family_once = None;
            for child_id in eligible {
                let person = people[child_id];
                let before = person.scalars.get(&reduction.child_value).ok_or_else(|| {
                    UnitDerivationError::InvalidPlan(format!(
                        "child `{child_id}` lacks reduction scalar `{}`",
                        reduction.child_value
                    ))
                })?;
                child_total.add_assign(
                    parse_decimal(
                        before,
                        &format!("child `{child_id}` scalar `{}`", reduction.child_value),
                    )?,
                    plan.decimal_precision,
                );
                let once_raw = person
                    .scalars
                    .get(&reduction.family_once_value)
                    .ok_or_else(|| {
                        UnitDerivationError::InvalidPlan(format!(
                            "child `{child_id}` lacks family-once scalar `{}`",
                            reduction.family_once_value
                        ))
                    })?;
                let once = parse_decimal(
                    once_raw,
                    &format!(
                        "child `{child_id}` scalar `{}`",
                        reduction.family_once_value
                    ),
                )?;
                match family_once {
                    None => family_once = Some(once),
                    Some(ref previous) if previous == &once => {}
                    Some(previous) => {
                        return Err(UnitDerivationError::InvalidPlan(format!(
                            "family-once scalar `{}` disagrees across children: {} != {}",
                            reduction.family_once_value,
                            previous.to_text(),
                            once.to_text()
                        )));
                    }
                }
            }
            let value = match reduction.operation {
                FamilyReductionOperation::SumChildrenThenSubtractFamilyOnce => child_total
                    .clone()
                    .subtract(family_once.unwrap_or_else(ExactDecimal::zero))
                    .round_significant(plan.decimal_precision)
                    .max_zero(),
            };
            scalars.insert(reduction.gross_output.clone(), decimal_text(child_total));
            scalars.insert(reduction.output.clone(), decimal_text(value));
        }

        families.push(AggregatedFamily {
            id: unit.id.clone(),
            members: unit.members.clone(),
            partner_present,
            scalars,
            counts,
            children,
        });
    }
    families.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(AggregationResult {
        schema: super::EXPERIMENTAL_AGGREGATION_PLAN_SCHEMA.to_string(),
        plan: plan.id.clone(),
        families,
        trace_roots: vec![derivation.trace.root],
    })
}

fn age_in_range(age: i64, minimum: Option<i64>, maximum: Option<i64>) -> bool {
    minimum.is_none_or(|minimum| age >= minimum) && maximum.is_none_or(|maximum| age <= maximum)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ExactDecimal {
    coefficient: BigInt,
    scale: u32,
}

impl ExactDecimal {
    fn zero() -> Self {
        Self {
            coefficient: BigInt::from(0),
            scale: 0,
        }
    }

    fn new(mut coefficient: BigInt, mut scale: u32) -> Self {
        while scale > 0 && (&coefficient % 10_u8) == BigInt::from(0) {
            coefficient /= 10_u8;
            scale -= 1;
        }
        Self { coefficient, scale }
    }

    fn ten_to(power: u32) -> BigInt {
        BigInt::from(10_u8).pow(power)
    }

    fn aligned_coefficient(&self, scale: u32) -> BigInt {
        &self.coefficient * Self::ten_to(scale - self.scale)
    }

    fn add_assign(&mut self, other: Self, precision: u32) {
        let scale = self.scale.max(other.scale);
        *self = Self::new(
            self.aligned_coefficient(scale) + other.aligned_coefficient(scale),
            scale,
        )
        .round_significant(precision);
    }

    fn subtract(self, other: Self) -> Self {
        let scale = self.scale.max(other.scale);
        Self::new(
            self.aligned_coefficient(scale) - other.aligned_coefficient(scale),
            scale,
        )
    }

    fn max_zero(self) -> Self {
        if self.coefficient < BigInt::from(0) {
            Self::zero()
        } else {
            self
        }
    }

    /// Apply a finite significant-digit context with round-half-even. The NZ
    /// comparison records 40 as an explicit reproducibility input.
    fn round_significant(self, precision: u32) -> Self {
        let negative = self.coefficient < BigInt::from(0);
        let absolute = if negative {
            -self.coefficient.clone()
        } else {
            self.coefficient.clone()
        };
        let digits = absolute.to_string().len() as u32;
        if digits <= precision {
            return self;
        }
        let dropped = digits - precision;
        let divisor = Self::ten_to(dropped);
        let mut quotient = &absolute / &divisor;
        let remainder = &absolute % &divisor;
        let doubled = remainder * 2_u8;
        let odd = (&quotient % 2_u8) != BigInt::from(0);
        if doubled > divisor || (doubled == divisor && odd) {
            quotient += 1_u8;
        }
        if negative {
            quotient = -quotient;
        }
        if dropped <= self.scale {
            Self::new(quotient, self.scale - dropped)
        } else {
            Self::new(quotient * Self::ten_to(dropped - self.scale), 0)
        }
    }

    fn to_text(&self) -> String {
        if self.coefficient == BigInt::from(0) {
            return "0".to_string();
        }
        let signed = self.coefficient.to_string();
        let (sign, digits) = signed
            .strip_prefix('-')
            .map_or(("", signed.as_str()), |digits| ("-", digits));
        if self.scale == 0 {
            return format!("{sign}{digits}");
        }
        let scale = self.scale as usize;
        if digits.len() <= scale {
            format!("{sign}0.{}{digits}", "0".repeat(scale - digits.len()))
        } else {
            let split = digits.len() - scale;
            format!("{sign}{}.{}", &digits[..split], &digits[split..])
        }
    }
}

fn parse_decimal(raw: &str, label: &str) -> Result<ExactDecimal, UnitDerivationError> {
    let raw = raw.trim();
    let (mantissa, exponent) = match raw.find(['e', 'E']) {
        Some(index) => {
            if raw[index + 1..].contains(['e', 'E']) {
                return Err(UnitDerivationError::InvalidPlan(format!(
                    "{label} is not a decimal: repeated exponent"
                )));
            }
            let exponent = raw[index + 1..].parse::<i64>().map_err(|error| {
                UnitDerivationError::InvalidPlan(format!(
                    "{label} is not a decimal: invalid exponent: {error}"
                ))
            })?;
            (&raw[..index], exponent)
        }
        None => (raw, 0),
    };
    let (negative, unsigned) = match mantissa.as_bytes().first() {
        Some(b'-') => (true, &mantissa[1..]),
        Some(b'+') => (false, &mantissa[1..]),
        _ => (false, mantissa),
    };
    let mut parts = unsigned.split('.');
    let integer = parts.next().unwrap_or_default();
    let fraction = parts.next().unwrap_or_default();
    if parts.next().is_some()
        || (integer.is_empty() && fraction.is_empty())
        || !integer
            .bytes()
            .chain(fraction.bytes())
            .all(|byte| byte.is_ascii_digit())
    {
        return Err(UnitDerivationError::InvalidPlan(format!(
            "{label} is not a decimal: invalid digits"
        )));
    }
    let digits = format!("{integer}{fraction}");
    let mut coefficient = BigInt::parse_bytes(digits.as_bytes(), 10).ok_or_else(|| {
        UnitDerivationError::InvalidPlan(format!("{label} is not a decimal: invalid coefficient"))
    })?;
    if negative {
        coefficient = -coefficient;
    }
    let scale = i64::try_from(fraction.len())
        .ok()
        .and_then(|fraction| fraction.checked_sub(exponent))
        .ok_or_else(|| {
            UnitDerivationError::InvalidPlan(format!("{label} is not a decimal: scale overflow"))
        })?;
    if scale < 0 {
        let power = u32::try_from(-scale).map_err(|_| {
            UnitDerivationError::InvalidPlan(format!("{label} is not a decimal: scale overflow"))
        })?;
        coefficient *= ExactDecimal::ten_to(power);
        Ok(ExactDecimal::new(coefficient, 0))
    } else {
        let scale = u32::try_from(scale).map_err(|_| {
            UnitDerivationError::InvalidPlan(format!("{label} is not a decimal: scale overflow"))
        })?;
        Ok(ExactDecimal::new(coefficient, scale))
    }
}

fn decimal_text(value: ExactDecimal) -> String {
    value.to_text()
}
