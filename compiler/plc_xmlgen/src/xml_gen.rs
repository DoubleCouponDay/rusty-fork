use std::{borrow::Cow, collections::HashMap, fs::{File, copy}, i32::MAX, io::Error, path::{Path, PathBuf}};

use super::serializer::*;

use plc_ast::ast::*;

use xml::{attribute::Attribute, common::XmlVersion, name::Name, namespace::Namespace, writer::XmlEvent, EmitterConfig, EventWriter};
use chrono::Local;

#[derive(Debug)]
pub struct GenerationParameters {
    pub output_xml_omron: bool    
}

impl GenerationParameters {
    pub fn new() -> Self {
        GenerationParameters { 
            output_xml_omron: false 
        }
    }
}

/// <?xml version=\"1.0\"?>
/// <Project xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xmlns:smcext=\"https://www.ia.omron.com/Smc\" xsi:schemaLocation=\"https://www.ia.omron.com/Smc IEC61131_10_Ed1_0_SmcExt1_0_Spc1_0.xsd\" schemaVersion=\"1\" xmlns=\"www.iec.ch/public/TC65SC65BWG7TF10\">
///     <FileHeader companyName=\"OMRON Corporation\" productName=\"Sysmac Studio\" productVersion=\"1.30.0.0\" />
///     <ContentHeader name=\"Sample\" creationDateTime="">
///     </ContentHeader>
///     <Types>
///         <GlobalNamespace>
///         </GlobalNamespace>
///     </Types>
///     <Instances>
///     </Instances>
/// </Project>
pub fn get_omron_template() -> Node {
    Node::new_str("Project")
        .attribute_str("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance")
        .attribute_str("xmlns:smcext", "https://www.ia.omron.com/Smc")
        .attribute_str("xsi:schemaLocation", OMRON_SCHEMA)
        .attribute_str("schemaVersion", "1")
        .attribute_str("xmlns", "www.iec.ch/public/TC65SC65BWG7TF10")
            .child(&SFileHeader::new()
                .attribute_str("companyName", "OMRON Corporation")
                .attribute_str("productName", "Sysmac Studio")
                .attribute_str("productVersion", "1.30.0.0"))
            .child(&SContentHeader::new()
                .attribute_str("name", "Sample")
                .attribute("creationDateTime".to_string(), Local::now().to_rfc3339()))
            .child(&STypes::new()
                .child(&SGlobalNamespace::new()))
            .child(&SInstances::new())
}

pub const OMRON_SCHEMA: &'static str = "https://www.ia.omron.com/Smc IEC61131_10_Ed1_0_SmcExt1_0_Spc1_0.xsd";

pub fn parse_project_into_nodetree(units: &Vec<&CompilationUnit>, schema_path: &'static str, output_path: &PathBuf, mut output_root: Node) -> Result<(), Error> {
    for a in 0..units.len() {
        let current_unit = units[a];
        let unit_name = current_unit.file.get_name().unwrap_or("");

        let _ = parse_globals(current_unit, unit_name, schema_path, &mut output_root);
        let _ = parse_custom_types(current_unit, &mut output_root);
        let _ = parse_pous(current_unit, unit_name, schema_path, &mut output_root);
    }
    write_xml_file(output_path, output_root)?;
    Ok(())
}

fn parse_custom_types(current_unit: &CompilationUnit, output_root: &mut Node) -> Result<(), ()> {
    let maybe_types_root: Option<&mut Node> = output_root.children.iter_mut().find(|a| a.name == TYPES);
    let types_root: &mut Node = maybe_types_root.ok_or(())?;    
    let maybe_global_root: Option<&mut Node> = types_root.children.iter_mut().find(|a| a.name == GLOBAL_NAMESPACE);
    let global_root: &mut Node = maybe_global_root.ok_or(())?;

    for a in 0..current_unit.user_types.len() {
        let current_usertype = &current_unit.user_types[a];

        let customtype_ready: Option<SDataTypeDecl> = match &current_usertype.data_type {
            DataType::StructType { name, variables } => { //STRUCT
                if name.is_none() { //every structure must have a name
                    continue;
                }
                let unwrapped_name = name.clone().unwrap();

                let mut spec_node = SUserDefinedTypeSpec::new()
                    .attribute_str("xsi:type", "StructTypeSpec");



                for b in 0..variables.len() {
                    let current_variable = &variables[b];
                    let maybe_typename = current_variable.data_type_declaration.get_name();

                    if maybe_typename.is_none() { //every variable must have a type
                        continue;
                    }
                    let typename = String::from(maybe_typename.unwrap());

                    let typename_node = STypeName::new()
                        .content(typename);

                    let type_node = SType::new()
                        .child(&typename_node);

                    let member_node = SMember::new()
                        .attribute(String::from("name"), current_variable.name.clone())
                        .child(&type_node);

                    spec_node = spec_node.child(&member_node);
                }

                if spec_node.inner().children.len() == 0 { //structs must have <Member> elements, otherwise delete it
                    None
                }

                else {
                    let decl_node1 = SDataTypeDecl::new()
                        .attribute(String::from("name"), unwrapped_name)
                        .child(&spec_node);

                    Some(decl_node1)
                }
            },
            DataType::EnumType { name, numeric_type, elements } => { //ENUM
                if name.is_none() { //every structure must have a name
                    continue;
                }
                let unwrapped_enum_type = name.clone().unwrap();

                let enumerators = match &elements.stmt {
                    AstStatement::ExpressionList(ast_nodes) => ast_nodes.iter().map(|a| {
                        match &a.stmt {
                            AstStatement::Assignment(assignment) => parse_enum_expression(assignment),
                            other => panic!("Expected Assignment. Instead got: {:?}", other)
                        }
                    }).collect(),

                    AstStatement::Assignment(assignment) => vec![parse_enum_expression(assignment)],
                    other => panic!("Expected ExpressionList or Assignment. Instead got: {:?}", other)
                };

                let base_node = SBaseType::new()
                    .content(numeric_type.clone());

                let formatted = format_enum_initials(enumerators);

                let spec_node = SUserDefinedTypeSpec::new()
                    .attribute_str("xsi:type", "EnumTypeWithNamedValueSpec")                    
                    .children(formatted)
                    .child(&base_node); //<BaseType> element must be declared below all the <Member> elements, apparently

                let decl_node2 = SDataTypeDecl::new()
                    .attribute(String::from("name"), unwrapped_enum_type)
                    .child(&spec_node);

                Some(decl_node2)
            },
            _ => None                                
        };

        if let Some(unwrapped_ready) = customtype_ready {
            global_root.child_borrowed(&unwrapped_ready);
        }        
    }
    Ok(())
}

fn parse_enum_expression(input: &Assignment) -> NameAndInitialValue {
    let enum_variant_name = match &input.left.stmt {
        AstStatement::ReferenceExpr(reference_exp) => {
            match &reference_exp.access {
                ReferenceAccess::Member(member_exp) => {
                    match &member_exp.stmt {
                        AstStatement::Identifier(name) => {
                            name.clone()
                        }
                        other => panic!("Expected identifier. Instead got: {:?}", other)
                    }
                }
                other => panic!("Expected Member. Instead got: {:?}", other)
            }
        },
        other => panic!("Expected ReferenceExpr. Instead got: {:?}", other)
    };

    let enum_variant_initialiser = match &input.right.stmt {
        AstStatement::Literal(literal) => literal.to_string(),
        AstStatement::BinaryExpression(binary_exp) => {
            match &binary_exp.right.stmt {
                AstStatement::Literal(literal) => literal.to_string(),
                other => panic!("Expected Literal. Instead got: {:?}", other)
            }
        }
        other => panic!("Expected LiteralInteger or BinaryExpression. Instead got: {:?}", other)
    };

    NameAndInitialValue {name: enum_variant_name, initial_value: enum_variant_initialiser}
}

struct NameAndInitialValue {
    pub name: String,
    pub initial_value: String
}

fn format_enum_initials(mut enum_variants: Vec<NameAndInitialValue>) -> Vec<Box<dyn IntoNode>> {
    let mut viewed_values: HashMap<String, ()> = HashMap::new(); // Own strings for ownership
    
    for i in 0..enum_variants.len() {
        let current_initial = &mut enum_variants[i].initial_value;
        
        if !viewed_values.contains_key(current_initial) {
            viewed_values.insert(current_initial.clone(), ());
            continue;
        }
        
        // Conflict: auto-increment
        let parsed_value = current_initial.parse::<i32>().expect("signed integer");
        let mut increment = 1;
        loop {
            let new_value = parsed_value.checked_add(increment).expect("no overflow");
            let new_str = new_value.to_string();
            if !viewed_values.contains_key(&new_str) {
                *current_initial = new_str;
                viewed_values.insert(current_initial.clone(), ());
                break;
            }
            increment += 1;
        }
    }
    
    enum_variants.into_iter().map(|a| {
        Box::new(SEnumerator::new()
            .attribute(String::from("name"), a.name)
            .attribute(String::from("value"), a.initial_value)) as Box<dyn IntoNode>
    }).collect()
}

fn parse_pous(current_unit: &CompilationUnit, unit_name: &str, schema_path: &'static str, output_root: &mut Node) -> Result<(), ()> {


        //Functions



        //Function blocks



        //Programs

    
    Ok(())
}

fn parse_globals(current_unit: &CompilationUnit, unit_name: &str, schema_path: &'static str, output_root: &mut Node) -> Result<(), ()> {
    let maybe_globals_root: Option<&mut Node> = output_root.children.iter_mut().find(|a| a.name == INSTANCES);
    let globals_root = maybe_globals_root.ok_or(())?;

    //create the 4 destinations for <GlobalVars>
    let mut constant_retain_globals = SGlobalVars::new()
        .attribute_str("constant", "true")
        .attribute_str("retain", "true");

    let mut constant_globals = SGlobalVars::new()
        .attribute_str("constant", "true");

    let mut retain_globals = SGlobalVars::new()
        .attribute_str("retain", "true");

    let mut normal_globals = SGlobalVars::new();

    //parse the unit into nodes
    for a in 0..current_unit.global_vars.len() {
        let current_global = &current_unit.global_vars[a];
        let mut parsed_variables: Vec<Box<dyn IntoNode>> = Vec::with_capacity(current_global.variables.len());

        for b in 0..current_global.variables.len() {
            let current_variable = &current_global.variables[b];

            let network_publish = match current_global.kind {
                VariableBlockType::Global(network_publish_mode) => network_publish_mode.to_string(),
                _ => NetworkPublishMode::DoNotPublish.to_string()
            };

            let additional_property_node = SOmronGlobalVariableAdditionalProperties::new()
                .attribute("networkPublish".to_string(), network_publish);

            let data_node = SOmronData::new() //<Data>
                .attribute_str("name", schema_path)
                .attribute_str("handleUnknown", "discard")
                .child(&additional_property_node);

            let adddata_node = SOmronAddData::new() //<AddData>
                .child(&data_node);

            let maybe_typename = current_variable.data_type_declaration.get_name();

            if maybe_typename.is_none() { //every variable needs a typename
                continue;
            }
            let typename = maybe_typename.unwrap().to_string();

            let typename_node = STypeName::new() //<TypeName>
                .content(typename);

            let type_node = SType::new() //<Type>
                .child(&typename_node);

            let mut new_var = SOmronVariable::new() //<Variable>
                .with_name(current_variable.name.clone())
                .child(&adddata_node)
                .child(&type_node);

            if let Some(variable_node) = &current_variable.initializer && let AstStatement::Literal(variable_value
            ) = &variable_node.stmt {
                let simple_node = SSimpleValue::new() //<SimpleValue />
                    .attribute(String::from("value"), variable_value.to_string())
                    .close();

                let initial_node = SInitialValue::new() //<InitialValue>
                    .child(&simple_node);

                new_var = new_var.child(&initial_node);
            }
            parsed_variables.push(Box::new(new_var));
        }

        //add globals to the correct element
        if current_global.constant && current_global.retain {
            constant_retain_globals = constant_retain_globals.children(parsed_variables);
        }

        else if current_global.constant {
            constant_globals = constant_globals.children(parsed_variables);
        }

        else if current_global.retain {
            retain_globals = retain_globals.children(parsed_variables);
        }

        else {
            normal_globals = normal_globals.children(parsed_variables);
        }
    }
    
    //relinquish copies of the nodes into the tree
    let name_label = String::from("name");
    let resources_name = format!("{}_{}", unit_name, RESOURCE);
    
    let resource_node = SResource::new()
        .attribute(name_label.clone(), resources_name)
        .attribute_str("resourceTypeName", "")
        .child(&constant_retain_globals)
        .child(&constant_globals)
        .child(&retain_globals)
        .child(&normal_globals);

    let config_name = format!("{}_{}", unit_name, CONFIGURATION);

    let configuration_node = SConfiguration::new()
        .attribute(name_label, config_name)
        .child(&resource_node);

    globals_root.child_borrowed(&configuration_node); //need to borrow a mut Node so I don't break the root nodes reference to the globals node
    return Ok(());
}

pub fn write_xml_file(output_path: &PathBuf, treenode: Node) -> Result<(), Error> {
    let file = File::create(output_path)?;

    let mut writer = EmitterConfig::new()
        .perform_indent(true)
        .create_writer(file);

    let top = XmlEvent::StartDocument {
        encoding: Some("UTF-8"),
        version: XmlVersion::Version10,
        standalone: None
    };

    let _ = writer.write(top).or_else(|a| {
        return Err(Error::new(std::io::ErrorKind::Other, a));
    });    

    return recurse_write_xml(&mut writer, output_path, treenode);
}

fn recurse_write_xml(writer: &mut EventWriter<File>, output_path: &PathBuf, mut treenode: Node) -> Result<(), Error> {
    //open the element
    let start = XmlEvent::StartElement {
        name: Name::from(treenode.name.as_str()),
        attributes: treenode.attributes.iter().map(|a| {
            Attribute {
                name: Name::from(a.0.as_str()),
                value: a.1.as_str()
            }
        })
        .collect(), 
        namespace: Cow::Owned(Namespace::empty())
    };

    let _ = writer.write(start).or_else(|a| {
        return Err(Error::new(std::io::ErrorKind::Other, a));
    });

    if let Some(content) = &treenode.content && treenode.children.len() == 0 {
        let content_event = XmlEvent::CData(content);

        let _ = writer.write(content_event).or_else(|a| {
            return Err(Error::new(std::io::ErrorKind::Other, a));
        });
    }

    //recurse through children
    for item in treenode.children.drain(0..) {
        recurse_write_xml(writer, output_path, item)?;
    }

    //close the element
    let end = XmlEvent::end_element();

    let _ = writer.write(end).or_else(|a| {
        return Err(Error::new(std::io::ErrorKind::Other, a));
    });
    Ok(())
}

pub fn copy_xmlfiles_to_output(temp_paths: Vec<&Path>, output_path: PathBuf) -> Result<PathBuf, Error> {
    if temp_paths.len() == 0 {
        return Ok(output_path);
    }
    let xml_file = temp_paths.iter().find(|a| { //grab the file which has the right name, although both xml duplicates have the same content
        if let Some(ext) = a.extension() && ext.to_ascii_uppercase() == "XML" {
            return true;
        }
        return false;
    })
    .unwrap(); 

    copy(xml_file, &output_path)?;
    Ok(output_path)
}
