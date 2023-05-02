use noctilucent::parser::condition::ConditionsParseTree;
use noctilucent::parser::lookup_table::MappingsParseTree;
use noctilucent::parser::lookup_table::{MappingInnerValue, MappingParseTree};
use noctilucent::parser::output::OutputsParseTree;
use noctilucent::parser::parameters::Parameters;
use noctilucent::parser::resource::IntrinsicFunction;
use noctilucent::parser::resource::{
    build_resources, ResourceParseTree, ResourceValue, ResourcesParseTree,
};
use noctilucent::primitives::WrapperF64;
use noctilucent::CloudformationParseTree;
use serde_yaml::Value;

mod json;

macro_rules! map(
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = ::std::collections::HashMap::new();
            $(
                m.insert($key.to_string(), $value);
            )+
            m
        }
     };
);

macro_rules! assert_resource_equal {
    ($val:expr, $resource:expr) => {
        let obj = ($val).as_mapping().unwrap();
        let resources = build_resources(obj).unwrap();
        assert_eq!(resources.resources[0], ($resource))
    };
}

macro_rules! assert_template_equal {
    ($val:expr, $cfn_tree:expr) => {{
        let cfn_template = CloudformationParseTree::build(&$val).unwrap();
        let cfn_tree = $cfn_tree;
        assert_eq!(cfn_template.parameters.params, cfn_tree.parameters.params);
        assert_eq!(cfn_template.mappings.mappings, cfn_tree.mappings.mappings);
        assert_eq!(cfn_template.outputs.outputs, cfn_tree.outputs.outputs);
        assert_eq!(cfn_template.logical_lookup, cfn_tree.logical_lookup);
        assert_eq!(
            cfn_template.conditions.conditions,
            cfn_tree.conditions.conditions
        );
        assert_eq!(
            cfn_template.resources.resources,
            cfn_tree.resources.resources
        );
    }};
}

#[test]
fn test_parse_tree_basics() {
    let a = json!({
        "LogicalResource": {
            "Type": "AWS::IAM::Role",
            "Properties": {
                "RoleName": "bob",
                "AssumeTime": 20,
                "Bool": true,
                "NotExistent": {"Ref": "AWS::NoValue"},
                "Array": ["hi", "there"]
            }
        }
    });

    let resource = ResourceParseTree {
        name: "LogicalResource".into(),
        condition: Option::None,
        metadata: Option::None,
        update_policy: Option::None,
        deletion_policy: Option::None,
        dependencies: vec![],
        resource_type: "AWS::IAM::Role".into(),
        properties: map! {
            "RoleName" => ResourceValue::String("bob".into()),
            "AssumeTime" => ResourceValue::Number(20),
            "Bool" => ResourceValue::Bool(true),
            "NotExistent" => ResourceValue::Null,
            "Array" => ResourceValue::Array(vec![ResourceValue::String("hi".into()), ResourceValue::String("there".into())])
        },
    };
    assert_resource_equal!(a, resource);
}

#[test]
fn test_basic_parse_tree_with_condition() {
    let a: Value = json!({
        "LogicalResource": {
            "Type": "AWS::IAM::Role",
            "Condition": "SomeCondition",
            "Properties": {
                "RoleName": "bob",
                "AssumeTime": 20,
                "Bool": true,
                "NotExistent": {"Ref": "AWS::NoValue"},
                "Array": ["hi", "there"]
            }
        }
    });

    let resource = ResourceParseTree {
        name: "LogicalResource".into(),
        condition: Option::Some("SomeCondition".into()),
        metadata: Option::None,
        update_policy: Option::None,
        deletion_policy: Option::None,
        dependencies: vec![],
        resource_type: "AWS::IAM::Role".into(),
        properties: map! {
            "RoleName" => ResourceValue::String("bob".into()),
            "AssumeTime" => ResourceValue::Number(20),
            "Bool" => ResourceValue::Bool(true),
            "NotExistent" => ResourceValue::Null,
            "Array" => ResourceValue::Array(vec![ResourceValue::String("hi".into()), ResourceValue::String("there".into())])
        },
    };
    assert_resource_equal!(a, resource);
}

#[test]
fn test_basic_parse_tree_with_metadata() {
    let a: Value = json!({
        "LogicalResource": {
            "Type": "AWS::IAM::Role",
            "Metadata": {
                "myArbitrary": "objectData"
            },
            "Properties": {
                "RoleName": "bob",
                "AssumeTime": 20,
                "Bool": true,
                "NotExistent": {"Ref": "AWS::NoValue"},
                "Array": ["hi", "there"]
            }
        }
    });

    let resource = ResourceParseTree {
        name: "LogicalResource".into(),
        condition: Option::None,
        metadata: Option::Some(ResourceValue::Object(map! {
            "myArbitrary" => ResourceValue::String("objectData".into())
        })),
        update_policy: Option::None,
        deletion_policy: Option::None,
        dependencies: vec![],
        resource_type: "AWS::IAM::Role".into(),
        properties: map! {
            "RoleName" => ResourceValue::String("bob".into()),
            "AssumeTime" => ResourceValue::Number(20),
            "Bool" => ResourceValue::Bool(true),
            "NotExistent" => ResourceValue::Null,
            "Array" => ResourceValue::Array(vec![ResourceValue::String("hi".into()), ResourceValue::String("there".into())])
        },
    };
    assert_resource_equal!(a, resource);
}

#[test]
fn test_parse_tree_basics_with_deletion_policy() {
    let a: Value = json!({
        "LogicalResource": {
            "Type": "AWS::IAM::Role",
            "DeletionPolicy": "Retain",
            "Properties": {
                "RoleName": "bob",
                "AssumeTime": 20,
                "Bool": true,
                "NotExistent": {"Ref": "AWS::NoValue"},
                "Array": ["hi", "there"]
            }
        }
    });

    let resource: ResourceParseTree = ResourceParseTree {
        name: "LogicalResource".into(),
        condition: Option::None,
        metadata: Option::None,
        update_policy: Option::None,
        deletion_policy: Option::Some("Retain".into()),
        dependencies: vec![],
        resource_type: "AWS::IAM::Role".into(),
        properties: map! {
            "RoleName" => ResourceValue::String("bob".into()),
            "AssumeTime" => ResourceValue::Number(20),
            "Bool" => ResourceValue::Bool(true),
            "NotExistent" => ResourceValue::Null,
            "Array" => ResourceValue::Array(vec![ResourceValue::String("hi".into()), ResourceValue::String("there".into())])
        },
    };

    assert_resource_equal!(a, resource);
}

#[test]
fn test_parse_tree_sub_str() {
    let a = json!({
        "LogicalResource": {
            "Type": "AWS::IAM::Role",
            "Properties": {
                "RoleName": {
                    "Fn::Sub": "bobs-role-${AWS::Region}"
                }
            }
        }
    });

    let resource = ResourceParseTree {
        name: "LogicalResource".into(),
        condition: Option::None,
        metadata: Option::None,
        update_policy: Option::None,
        deletion_policy: Option::None,
        dependencies: vec![],
        resource_type: "AWS::IAM::Role".into(),
        properties: map! {
            "RoleName" => IntrinsicFunction::Sub{ string:"bobs-role-${AWS::Region}".into(), replaces: None }.into()
        },
    };
    assert_resource_equal!(a, resource);
}

#[test]
fn test_parse_tree_yaml_codes() {
    let a = json!({
        "LogicalResource": {
            "Type": "AWS::IAM::Role",
            "Properties": {
                "RoleName": {
                    "!Sub": "bobs-role-${AWS::Region}"
                }
            }
        }
    });

    let resource = ResourceParseTree {
        name: "LogicalResource".into(),
        condition: Option::None,
        metadata: Option::None,
        update_policy: Option::None,
        deletion_policy: Option::None,
        dependencies: vec![],
        resource_type: "AWS::IAM::Role".into(),
        properties: map! {
            "RoleName" => IntrinsicFunction::Sub{ string: "bobs-role-${AWS::Region}".into(), replaces: None }.into()
        },
    };
    assert_resource_equal!(a, resource);
}
#[test]
fn test_parse_get_attr_shorthand() {
    let a = json!({
        "LogicalResource": {
            "Type": "AWS::IAM::Role",
            "Properties": {
                "RoleName": {
                    "Fn::GetAtt": "Foo.Bar"
                }
            }
        }
    });

    let resource = ResourceParseTree {
        name: "LogicalResource".into(),
        condition: Option::None,
        metadata: Option::None,
        update_policy: Option::None,
        deletion_policy: Option::None,
        dependencies: vec![],
        resource_type: "AWS::IAM::Role".into(),
        properties: map! {
            "RoleName" => IntrinsicFunction::GetAtt{logical_name:"Foo".into(), attribute_name:"Bar".into()}.into()
        },
    };
    assert_resource_equal!(a, resource);
}

#[test]
fn test_parse_tree_sub_list() {
    let a = json!({
        "LogicalResource": {
            "Type": "AWS::IAM::Role",
            "Properties": {
                "RoleName": {
                    "Fn::Sub": [
                        "bobs-role-${Region}",
                        {
                            "Region": {
                               "Ref": "AWS::Region"
                            }
                        }
                    ]
                }
            }
        }
    });

    let resource = ResourceParseTree {
        name: "LogicalResource".into(),
        condition: Option::None,
        resource_type: "AWS::IAM::Role".into(),
        metadata: Option::None,
        update_policy: Option::None,
        deletion_policy: Option::None,
        dependencies: vec![],
        properties: map! {
            "RoleName" => IntrinsicFunction::Sub{
                string: "bobs-role-${Region}".into(),
                replaces: Some(ResourceValue::Object(map!{
                    "Region" =>  IntrinsicFunction::Ref("AWS::Region".into()).into()
                }))
            }.into()
        },
    };
    assert_resource_equal!(a, resource);
}

#[test]
fn test_parse_simple_json_template() {
    let cfn_template = json!({
        "Resources": {
            "EC2Instance": {
                "Type": "AWS::EC2::Instance",
                "Properties": {
                    "ImageId": "ami-0c55b159cbfafe1f0",
                    "InstanceType": "t2.micro",
                    "KeyName": "my-key-pair",
                    "BlockDeviceMappings": [
                    {
                        "DeviceName": "/dev/xvda",
                        "Ebs": {
                            "VolumeSize": 8,
                            "VolumeType": "gp2"
                        }
                    }
                    ]
                }
            },
            "EBSVolume": {
                "Type": "AWS::EC2::Volume",
                "Properties": {
                    "Size": 10,
                    "AvailabilityZone": "us-east-1a",
                    "VolumeType": "gp2"
                }
            },
            "VolumeAttachment": {
                "Type": "AWS::EC2::VolumeAttachment",
                "Properties": {
                    "InstanceId": null,
                    "VolumeId": null,
                    "Device": "/dev/xvdf"
                }
            }
        }
    });

    let resources = ResourcesParseTree {
        resources: vec![
            ResourceParseTree {
                name: "EC2Instance".into(),
                condition: Option::None,
                resource_type: "AWS::EC2::Instance".into(),
                metadata: Option::None,
                update_policy: Option::None,
                deletion_policy: Option::None,
                dependencies: vec![],
                properties: map! {
                    "ImageId" => ResourceValue::String("ami-0c55b159cbfafe1f0".into()),
                    "InstanceType" => ResourceValue::String("t2.micro".into()),
                    "KeyName" => ResourceValue::String("my-key-pair".into()),
                    "BlockDeviceMappings" => ResourceValue::Array(vec![
                        ResourceValue::Object(map!{
                            "DeviceName" => ResourceValue::String("/dev/xvda".into()),
                            "Ebs" => ResourceValue::Object(map!{
                                "VolumeSize" => ResourceValue::Number(8),
                                "VolumeType" => ResourceValue::String("gp2".into())
                            })
                        })
                    ])
                },
            },
            ResourceParseTree {
                name: "EBSVolume".into(),
                condition: Option::None,
                resource_type: "AWS::EC2::Volume".into(),
                metadata: Option::None,
                update_policy: Option::None,
                deletion_policy: Option::None,
                dependencies: vec![],
                properties: map! {
                    "Size" => ResourceValue::Number(10),
                    "AvailabilityZone" => ResourceValue::String("us-east-1a".into()),
                    "VolumeType" => ResourceValue::String("gp2".into())
                },
            },
            ResourceParseTree {
                name: "VolumeAttachment".into(),
                condition: Option::None,
                resource_type: "AWS::EC2::VolumeAttachment".into(),
                metadata: Option::None,
                update_policy: Option::None,
                deletion_policy: Option::None,
                dependencies: vec![],
                properties: map! {
                    "InstanceId" => ResourceValue::Null,
                    "VolumeId" => ResourceValue::Null,
                    "Device" => ResourceValue::String("/dev/xvdf".into())
                },
            },
        ],
    };

    let cfn_tree = CloudformationParseTree {
        parameters: Parameters::new(),
        mappings: MappingsParseTree::new(),
        conditions: ConditionsParseTree::new(),
        logical_lookup: CloudformationParseTree::build_logical_lookup(&resources),
        resources,
        outputs: OutputsParseTree::new(),
    };

    assert_template_equal!(cfn_template, cfn_tree)
}

#[test]
fn test_parse_tree_with_fnfindinmap() {
    let cfn_template = json!(
        {
            "Resources": {
                "MyInstance": {
                    "Type": "AWS::EC2::Instance",
                    "Properties": {
                        "InstanceType": { "Fn::FindInMap": [ "InstanceTypes", { "Ref": "Region" }, "t2.micro" ] },
                        "ImageId": { "Fn::FindInMap": [ "AMIIds", { "Ref": "Region" }, "AmazonLinuxAMI" ] }
                    }
                }
            },
            "Mappings": {
                "InstanceTypes": {
                    "us-east-1": {
                        "t2.micro": "t2.micro",
                        "t2.small": "t2.small"
                    },
                    "us-west-2": {
                        "t2.micro": "t2.nano",
                        "t2.small": "t2.micro"
                    }
                },
                "AMIIds": {
                    "us-east-1": {
                        "AmazonLinuxAMI": "ami-0ff8a91507f77f867",
                        "UbuntuAMI": "ami-0c55b159cbfafe1f0"
                    },
                    "us-west-2": {
                        "AmazonLinuxAMI": "ami-0323c3dd2da7fb37d",
                        "UbuntuAMI": "ami-0bdb1d6c15a40392c"
                    }
                }
            }
        }

    );

    let resources = ResourcesParseTree {
        resources: vec![ResourceParseTree {
            name: "MyInstance".into(),
            condition: Option::None,
            resource_type: "AWS::EC2::Instance".into(),
            metadata: Option::None,
            update_policy: Option::None,
            deletion_policy: Option::None,
            dependencies: vec![],
            properties: map! {
                "InstanceType" => IntrinsicFunction::FindInMap{
                    map_name:ResourceValue::String("InstanceTypes".into()),
                    top_level_key:IntrinsicFunction::Ref("Region".into()).into(),
                    second_level_key:ResourceValue::String("t2.micro".into()),
                }.into(),
                "ImageId" => IntrinsicFunction::FindInMap{
                    map_name:ResourceValue::String("AMIIds".into()),
                    top_level_key:IntrinsicFunction::Ref("Region".into()).into(),
                    second_level_key: ResourceValue::String("AmazonLinuxAMI".into()),
                }.into()
            },
        }],
    };

    let cfn_tree = CloudformationParseTree {
        parameters: Parameters::new(),
        mappings: MappingsParseTree {
            mappings: map! {
                "InstanceTypes" => MappingParseTree {
                    mappings: map! {
                            "us-east-1" => map! {
                                "t2.micro" => MappingInnerValue::String("t2.micro".into()),
                                "t2.small" => MappingInnerValue::String("t2.small".into())
                            },
                            "us-west-2" => map! {
                                "t2.micro" => MappingInnerValue::String("t2.nano".into()),
                                "t2.small" => MappingInnerValue::String("t2.micro".into())
                            }
                    },
                },
                "AMIIds" => MappingParseTree {
                    mappings: map! {
                            "us-east-1" => map! {
                                "AmazonLinuxAMI" => MappingInnerValue::String("ami-0ff8a91507f77f867".into()),
                                "UbuntuAMI" => MappingInnerValue::String("ami-0c55b159cbfafe1f0".into())
                            },
                            "us-west-2" => map! {
                                "AmazonLinuxAMI" => MappingInnerValue::String("ami-0323c3dd2da7fb37d".into()),
                                "UbuntuAMI" => MappingInnerValue::String("ami-0bdb1d6c15a40392c".into())
                            }
                    },
                }
            },
        },
        conditions: ConditionsParseTree::new(),
        logical_lookup: CloudformationParseTree::build_logical_lookup(&resources),
        resources,
        outputs: OutputsParseTree::new(),
    };

    assert_template_equal!(cfn_template, cfn_tree)
}

#[test]
fn test_parse_tree_resource_with_floats() {
    let a = json!({
        "Alarm": {
            "Type": "AWS::CloudWatch::Alarm",
            "Properties": {
                "ComparisonOperator": "GreaterThanOrEqualToThreshold",
                "AlarmName": {
                    "Fn::Sub": [
                        "${Tag}-FrontendDistributedCacheTrafficImbalanceAlarm",
                        {
                            "Tag": {
                               "Ref": "AWS::Region"
                            }
                        }
                    ]
                },
                "Threshold": 3.5
            }
        }
    });

    let resource = ResourceParseTree {
        name: "Alarm".into(),
        condition: Option::None,
        resource_type: "AWS::CloudWatch::Alarm".into(),
        metadata: Option::None,
        update_policy: Option::None,
        deletion_policy: Option::None,
        dependencies: vec![],
        properties: map! {
            "AlarmName" => IntrinsicFunction::Sub{
                string: "${Tag}-FrontendDistributedCacheTrafficImbalanceAlarm".into(),
                replaces: Some(ResourceValue::Object(map!{
                    "Tag" =>  IntrinsicFunction::Ref("AWS::Region".into()).into()
                }))
            }.into(),
            "ComparisonOperator" => ResourceValue::String("GreaterThanOrEqualToThreshold".to_string()),
            "Threshold" => ResourceValue::Double(WrapperF64::new(3.5))
        },
    };
    assert_resource_equal!(a, resource);
}
