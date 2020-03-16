use crate::errors::*;


use std::collections::HashMap;

use crate::{proto, base};
use crate::hashmap;
use crate::components::{Component, Accuracy, Expandable, Report};


use crate::base::{NodeProperties, Value, ValueProperties};
use crate::utilities::json::JSONRelease;


impl Component for proto::DpCovariance {
    // modify min, max, n, categories, is_public, non-null, etc. based on the arguments and component
    fn propagate_property(
        &self,
        _privacy_definition: &proto::PrivacyDefinition,
        _public_arguments: &HashMap<String, Value>,
        properties: &base::NodeProperties,
    ) -> Result<ValueProperties> {
        Err("DPCovariance is ethereal, and has no property propagation".into())
    }

    fn get_names(
        &self,
        _properties: &NodeProperties,
    ) -> Result<Vec<String>> {
        Err("get_names not implemented".into())
    }
}


impl Expandable for proto::DpCovariance {
    fn expand_component(
        &self,
        _privacy_definition: &proto::PrivacyDefinition,
        component: &proto::Component,
        _properties: &base::NodeProperties,
        component_id: u32,
        maximum_id: u32,
    ) -> Result<(u32, HashMap<u32, proto::Component>)> {
        let mut current_id = maximum_id.clone();
        let mut graph_expansion: HashMap<u32, proto::Component> = HashMap::new();

        // covariance
        current_id += 1;
        let id_covariance = current_id.clone();
        graph_expansion.insert(id_covariance, proto::Component {
            arguments: hashmap![
                "left".to_owned() => *component.arguments.get("left").unwrap(),
                "right".to_owned() => *component.arguments.get("right").unwrap()
            ],
            variant: Some(proto::component::Variant::from(proto::Covariance {})),
            omit: true,
            batch: component.batch,
        });

        // noise
        graph_expansion.insert(component_id, proto::Component {
            arguments: hashmap!["data".to_owned() => id_covariance],
            variant: Some(proto::component::Variant::from(proto::LaplaceMechanism {
                privacy_usage: self.privacy_usage.clone()
            })),
            omit: false,
            batch: component.batch,
        });

        Ok((current_id, graph_expansion))
    }
}

impl Accuracy for proto::DpCovariance {
    fn accuracy_to_privacy_usage(
        &self,
        _privacy_definition: &proto::PrivacyDefinition,
        _properties: &base::NodeProperties,
        _accuracy: &proto::Accuracy,
    ) -> Option<proto::PrivacyUsage> {
        None
    }

    fn privacy_usage_to_accuracy(
        &self,
        _privacy_definition: &proto::PrivacyDefinition,
        _property: &base::NodeProperties,
    ) -> Option<f64> {
        None
    }
}

impl Report for proto::DpCovariance {
    fn summarize(
        &self,
        _node_id: &u32,
        _component: &proto::Component,
        _public_arguments: &HashMap<String, Value>,
        _properties: &NodeProperties,
        _release: &Value
    ) -> Result<Option<Vec<JSONRelease>>> {
        Ok(None)
    }
}