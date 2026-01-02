#![allow(clippy::new_without_default)]

use rustc_hash::FxHashMap;

#[derive(Clone)]
pub struct Node {
    pub name: String,
    pub children: Vec<Node>,

    /// XML attributes, e.g. `<position x="1">` where `x` is the attribute
    ///
    /// Design Note: We use a HashMap here to avoid duplicates but also update existing values in case of
    /// repeated function calls, e.g. `with_attribute("x", 1)` and `with_attribute("x", 2)` where the value of
    /// x has been updated from 1 to 2.
    pub attributes: FxHashMap<String, String>,

    /// Indicates if an element has a closed form, e.g. `<position x="1" y="2"/>`
    pub closed: bool,

    /// Indicates if an element has some text wrapped inside itself, e.g. `<expression>a + b</expression>`
    pub content: Option<String>,
}

pub trait IntoNode {
    fn inner(&self) -> Node;
}

impl Node {
    pub fn new(name: String) -> Self {
        Self { name, attributes: FxHashMap::default(), children: Vec::new(), closed: false, content: None }
    }

    pub fn new_str(name: &'static str) -> Self {
        Self { name: name.to_string(), attributes: FxHashMap::default(), children: Vec::new(), closed: false, content: None }
    }

    pub fn attribute(mut self, key: String, value: String) -> Self {
        self.attributes.insert(key, value);
        self
    }

    pub fn attribute_str(mut self, key: &'static str, value: &'static str) -> Self {
        self.attributes.insert(key.to_string(), value.to_string());
        self
    }    

    pub fn child(mut self, node: &dyn IntoNode) -> Self {
        self.children.push(node.inner());
        self
    }

    pub fn child_borrowed(&mut self, node: &dyn IntoNode) -> &Self {
        self.children.push(node.inner());
        self
    }

    pub fn children(mut self, nodes: Vec<&dyn IntoNode>) -> Self {
        self.children.extend(nodes.into_iter().map(IntoNode::inner));
        self
    }

    pub fn close(mut self) -> Self {
        self.closed = true;
        self
    }

    pub fn indent(level: usize) -> String {
        " ".repeat(level * 4)
    }

    fn serialize_content(indent: String, name: String, content: String) -> String {
        format!("{indent}<{name}>{content}</{name}>\n")
    }

    #[allow(unused_assignments)]
    pub fn serialize(&self, level: usize) -> String {
        let (name, indent) = (self.name.clone(), Node::indent(level));
        let attributes = self.attributes.iter().map(|(key, value)| format!("{key}=\"{value}\""));
        let attributes_str = attributes.collect::<Vec<_>>().join(" ");
        let mut result = String::new();

        if self.closed {
            return format!("{indent}<{name} {attributes_str}/>\n");
        }

        if let Some(content) = self.content.clone() {
            return Node::serialize_content(indent.to_string(), name, content);
        }

        result = format!("{indent}<{name} {attributes_str}>\n");
        self.children.iter().for_each(|child| result = format!("{result}{}", child.serialize(level + 1)));
        result = format!("{result}{indent}</{name}>\n");

        result
    }
}

macro_rules! newtype_impl {
    ($name_struct:ident, $name_node:expr, $negatable:expr) => {
        pub struct $name_struct(Node);

        impl IntoNode for $name_struct {
            fn inner(&self) -> Node {
                self.0.clone()
            }
        }

        impl $name_struct {
            pub fn new() -> Self {
                match $negatable {
                    true => Self(Node::new_str($name_node).attribute_str("negated", "false")),
                    false => Self(Node::new_str($name_node)),
                }
            }

            pub fn id(local_id: i32) -> Self {
                let new = $name_struct::new();
                new.with_id(local_id)
            }

            pub fn attribute(self, key: String, value: String) -> Self {
                Self(self.inner().attribute(key, value))
            }

            pub fn attribute_str(self, key: &'static str, value: &'static str) -> Self {
                Self(self.inner().attribute_str(key, value))
            }            

            pub fn maybe_attribute(self, key: String, value: Option<String>) -> Self {
                match value {
                    Some(value) => Self(self.inner().attribute(key, value)),
                    None => self,
                }
            }

            pub fn child(self, node: &dyn IntoNode) -> Self {
                Self(self.inner().child(node))
            }

            pub fn children(self, nodes: Vec<&dyn IntoNode>) -> Self {
                Self(self.inner().children(nodes))
            }

            pub fn serialize(self) -> String {
                self.inner().serialize(0)
            }

            pub fn with_id<T: std::fmt::Display>(self, id: T) -> Self {
                self.attribute_str("localId", Box::leak(id.to_string().into_boxed_str()))
            }

            pub fn with_ref_id<T: std::fmt::Display>(self, id: T) -> Self {
                self.attribute_str("refLocalId", Box::leak(id.to_string().into_boxed_str()))
            }

            pub fn with_execution_id<T: std::fmt::Display>(self, id: T) -> Self {
                self.attribute_str("executionOrderId", Box::leak(id.to_string().into_boxed_str()))
            }

            pub fn close(self) -> Self {
                Self(self.inner().close())
            }
        }
    };
}

// newtype_impl!(<struct name>, <xml name>, <is negatable>)
newtype_impl!(SInVariable, "inVariable", true);
newtype_impl!(SOutVariable, "outVariable", true);
newtype_impl!(SInOutVariable, "inOutVariable", true);
newtype_impl!(SInterface, "interface", false);
newtype_impl!(SLocalVars, "localVars", false);
newtype_impl!(SAddData, "addData", false);
newtype_impl!(SData, "data", false);
newtype_impl!(STextDeclaration, "textDeclaration", false);
newtype_impl!(SContent, "content", false);
newtype_impl!(SPosition, "position", false);
newtype_impl!(SConnectionPointIn, "connectionPointIn", false);
newtype_impl!(SConnectionPointOut, "connectionPointOut", false);
newtype_impl!(SRelPosition, "relPosition", false);
newtype_impl!(SConnection, "connection", false);
newtype_impl!(SBlock, "block", false);
newtype_impl!(SBody, "body", false);
newtype_impl!(SPou, "pou", false);
newtype_impl!(SInputVariables, "inputVariables", false);
newtype_impl!(SOutputVariables, "outputVariables", false);
newtype_impl!(SInOutVariables, "inOutVariables", false);
newtype_impl!(SVariable, "variable", true);
newtype_impl!(YFbd, "FBD", false);
newtype_impl!(SExpression, "expression", false);
newtype_impl!(SReturn, "return", false);
newtype_impl!(SNegate, "negated", false);
newtype_impl!(SConnector, "connector", false);
newtype_impl!(SContinuation, "continuation", false);
newtype_impl!(SJump, "jump", false);
newtype_impl!(SLabel, "label", false);
newtype_impl!(SAction, "action", false);
newtype_impl!(SActions, "actions", false);
newtype_impl!(SFileHeader, "FileHeader", false);
newtype_impl!(SContentHeader, "ContentHeader", false);
newtype_impl!(STypes, "Types", false);

impl SInVariable {
    pub fn connect(mut self, ref_local_id: i32) -> Self {
        self = self.child(&SConnectionPointIn::new().child(&SConnection::new().with_ref_id(ref_local_id)));
        self
    }

    pub fn with_expression(self, expression: String) -> Self {
        self.child(&SExpression::expression(expression))
    }
}

impl SOutVariable {
    pub fn connect(mut self, ref_local_id: i32) -> Self {
        self = self
            .child(&SConnectionPointIn::new().child(&SConnection::new().with_ref_id(ref_local_id).close()));
        self
    }

    pub fn connect_name(mut self, ref_local_id: i32, name: String) -> Self {
        self =
            self.child(&SConnectionPointIn::new().child(
                &SConnection::new().with_ref_id(ref_local_id).attribute("formalParameter".to_string(), name).close(),
            ));
        self
    }

    pub fn with_expression(self, expression: String) -> Self {
        self.child(&SExpression::expression(expression))
    }
}

impl SInOutVariable {
    pub fn with_expression(self, expression: String) -> Self {
        self.child(&SExpression::expression(expression))
    }
}

impl SReturn {
    pub fn init(local_id: i32, execution_order: i32) -> Self {
        Self::new().with_id(local_id).with_execution_id(execution_order)
    }

    pub fn connect(self, ref_local_id: i32) -> Self {
        self.child(&SConnectionPointIn::new().child(&SConnection::new().with_ref_id(ref_local_id)))
    }

    pub fn negate(self, value: bool) -> Self {
        self.child(&SAddData::new().child(&SData::new().child(
            &SNegate::new().attribute("value".to_string(), value.to_string()).close(),
        )))
    }
}

impl SContent {
    pub fn with_declaration(mut self, content: String) -> Self {
        self.0.content = Some(content);
        self
    }
}

impl SPou {
    pub fn init(name: String, kind: String, declaration: String) -> Self {
        Self::new()
            .attribute_str("xmlns", "http://www.plcopen.org/xml/tc6_0201")
            .attribute("name".to_string(), name)
            .attribute("pouType".to_string(), kind)
            .child(&SInterface::new().children(vec![
                    &SLocalVars::new().close(),
                    &SAddData::new().child(
                        &SData::new()
                            .attribute_str("name", "www.bachmann.at/plc/plcopenxml")
                            .attribute_str("handleUnknown", "implementation")
                            .child(
                                &STextDeclaration::new()
                                    .child(&SContent::new().with_declaration(declaration)),
                            ),
                    ),
                ]))
    }

    /// Implicitly wraps the fbd in a block node, i.e. <block><fbd>...<fbd/><block/>
    pub fn with_fbd(self, children: Vec<&dyn IntoNode>) -> Self {
        self.child(&SBody::new().child(&YFbd::new().children(children)))
    }

    pub fn with_actions(self, children: Vec<&dyn IntoNode>) -> Self {
        self.child(&SActions::new().children(children))
    }
}

impl SBlock {
    pub fn init(name: String, local_id: i32, execution_order_id: i32) -> Self {
        Self::new().with_name(name).with_id(local_id).with_execution_id(execution_order_id)
    }

    pub fn with_name(self, name: String) -> Self {
        self.attribute("typeName".to_string(), name)
    }

    pub fn with_input(self, variables: Vec<&dyn IntoNode>) -> Self {
        self.child(&SInputVariables::new().children(variables))
    }

    pub fn with_output(self, variables: Vec<&dyn IntoNode>) -> Self {
        self.child(&SOutputVariables::new().children(variables))
    }

    pub fn with_inout(self, variables: Vec<&dyn IntoNode>) -> Self {
        self.child(&SInOutVariables::new().children(variables))
    }
}

impl SBody {
    pub fn with_fbd(self, children: Vec<&dyn IntoNode>) -> Self {
        Self::new().child(&YFbd::new().children(children))
    }
}

impl SInputVariables {
    pub fn with_variables(variables: Vec<&dyn IntoNode>) -> Self {
        Self::new().children(variables)
    }
}

impl SOutputVariables {
    pub fn with_variables(variables: Vec<&dyn IntoNode>) -> Self {
        Self::new().children(variables)
    }
}

impl SVariable {
    pub fn with_name(self, name: String) -> Self {
        self.attribute("formalParameter".to_string(), name)
    }

    pub fn connect(self, ref_local_id: i32) -> Self {
        self.child(&SConnectionPointIn::new().child(&SConnection::new().with_ref_id(ref_local_id).close()))
    }

    pub fn connect_out(self, ref_local_id: i32) -> Self {
        self.child(&SConnectionPointOut::new().child(&SConnection::new().with_ref_id(ref_local_id).close()))
    }
}

impl SExpression {
    pub fn expression(expression: String) -> Self {
        let mut node = Self::new();
        node.0.content = Some(expression);
        node
    }
}

impl SConnector {
    pub fn with_name(self, name: String) -> Self {
        self.attribute("name".to_string(), name)
    }

    pub fn connect(self, ref_local_id: i32) -> Self {
        self.child(&SConnectionPointIn::new().child(&SConnection::new().with_ref_id(ref_local_id).close()))
    }
}

impl SContinuation {
    pub fn with_name(self, name: String) -> Self {
        self.attribute("name".to_string(), name)
    }

    pub fn connect_out(self, ref_local_id: i32) -> Self {
        self.child(&SConnectionPointOut::new().child(&SConnection::new().with_ref_id(ref_local_id).close()))
    }
}

impl SLabel {
    pub fn with_name(self, name: String) -> Self {
        self.attribute("label".to_string(), name)
    }
}

impl SJump {
    pub fn with_name(self, name: String) -> Self {
        self.attribute("label".to_string(), name)
    }

    pub fn connect(self, ref_local_id: i32) -> Self {
        self.child(&SConnectionPointIn::new().child(&SConnection::new().with_ref_id(ref_local_id).close()))
    }

    pub fn negate(self) -> Self {
        self.child(
            &SAddData::new().child(&SData::new().child(&SNegate::new().attribute("value".to_string(), "true".to_string()).close())),
        )
    }
}

impl SAction {
    pub fn name(name: String) -> Self {
        Self::new().attribute("name".to_string(), name)
    }

    pub fn with_fbd(self, children: Vec<&dyn IntoNode>) -> Self {
        self.child(&SBody::new().child(&YFbd::new().children(children)))
    }
}

//Omron specific xml
newtype_impl!(SGlobalNamespace, GLOBAL_NAMESPACE, false);
newtype_impl!(SInstances, INSTANCES, false);
newtype_impl!(SConfiguration, CONFIGURATION, false);
newtype_impl!(SResource, RESOURCE, false);
newtype_impl!(SGlobalVars, GLOBAL_VARS, false);

pub const GLOBAL_NAMESPACE: &'static str = "GlobalNamespace";
pub const INSTANCES: &'static str = "Instances";
pub const CONFIGURATION: &'static str = "Configuration";
pub const RESOURCE: &'static str = "Resource";
pub const GLOBAL_VARS: &'static str = "GlobalVars";
