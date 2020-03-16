use crate::errors::*;


use std::collections::HashMap;

use crate::{proto, base};

use crate::components::{Component, Aggregator, sensitivity_propagation_wrapper};
use crate::base::{Value, NodeProperties, AggregatorProperties, SensitivityType, prepend, ValueProperties, Sensitivity, ArrayNDProperties, Hashmap, ArrayND, HashmapProperties};
use itertools::Itertools;

impl Component for proto::Mean {
    fn propagate_property(
        &self,
        _privacy_definition: &proto::PrivacyDefinition,
        _public_arguments: &HashMap<String, Value>,
        properties: &base::NodeProperties,
    ) -> Result<ValueProperties> {
        let mut data_property = properties.get("data")
            .ok_or("data: missing")?.get_arraynd()
            .map_err(prepend("data:"))?.clone();

        // save a snapshot of the state when aggregating
        data_property.aggregator = Some(AggregatorProperties {
            component: proto::component::Variant::from(self.clone()),
            properties: properties.clone()
        });

        data_property.num_records = Some(1);

        Ok(data_property.into())
    }

    fn get_names(
        &self,
        _properties: &NodeProperties,
    ) -> Result<Vec<String>> {
        Err("get_names not implemented".into())
    }
}

impl Aggregator for proto::Mean {
    fn compute_sensitivity(
        &self,
        privacy_definition: &proto::PrivacyDefinition,
        properties: &NodeProperties,
        sensitivity_type: &SensitivityType
    ) -> Result<Sensitivity> {
        let mut data_property = properties.get("data")
            .ok_or("data: missing")?.clone();

        sensitivity_propagation_wrapper(
            privacy_definition, data_property, sensitivity_type,
            &arraynd_sensitivity,
            &hashmap_sensitivity,
            &aggregate_sensitivity
        )
    }
}


// given properties for a 2d array, return the sensitivities for every element in the resulting row vector
pub fn arraynd_sensitivity(
    _privacy_definition: &proto::PrivacyDefinition,
    sensitivity_type: &SensitivityType,
    data_property: &ArrayNDProperties,
) -> Result<Sensitivity> {

    data_property.assert_non_null()?;

    match sensitivity_type {
        SensitivityType::KNorm(k) => {
            if k != &1 {
                return Err("Mean sensitivity is only implemented for KNorm of 1".into())
            }
            let min = data_property.get_min_f64()?;
            let max = data_property.get_max_f64()?;
            let num_records = data_property.get_num_records()? as f64;

            Ok(vec![min.iter()
                .zip(max)
                .map(|(min, max)| (max - *min) / num_records)
                .collect()])
        },
        _ => return Err("Mean sensitivity is only implemented for KNorm of 1".into())
    }
}

// given properties for a hashmap of 2d arrays, return sensitivities for every element in the resulting aggregated matrix
pub fn hashmap_sensitivity(
    privacy_definition: &proto::PrivacyDefinition,
    sensitivity_type: &SensitivityType,
    hashmap_property: &HashmapProperties,
) -> Result<Sensitivity> {

    hashmap_property.assert_is_disjoint()?;
    hashmap_property.assert_is_not_columnar()?;

    hashmap_property.properties.get_values().iter()
        .map(|prop| Ok(arraynd_sensitivity(privacy_definition, sensitivity_type, prop.get_arraynd()?)?[0].clone()))
        .collect::<Result<Sensitivity>>()
}

// given properties for an aggregation of (likely partitioned) data, return the sensitivities after aggregating the aggregations
pub fn aggregate_sensitivity(
    privacy_definition: &proto::PrivacyDefinition,
    sensitivity_type: &SensitivityType,
    aggregator: &AggregatorProperties,
) -> Result<Sensitivity> {
    let partition_sensitivities: Sensitivity = aggregator.component
        .compute_sensitivity(privacy_definition, &aggregator.properties, sensitivity_type)?;
    let row_sensitivity: Vec<f64> = partition_sensitivities.into_iter()
        .fold1(|row_a: Vec<f64>, row_b: Vec<f64>| row_a.iter().zip(row_b)
            .map(|(cell_a, cell_b)| cell_a.max(cell_b)).collect())
        .unwrap_or(Vec::new());

    Ok(vec![row_sensitivity.clone()])
}