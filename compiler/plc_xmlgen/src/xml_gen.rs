use std::{borrow::Cow, fs::{copy, File}, io::Error, path::{Path, PathBuf}};

use super::serializer::*;
use chrono::Local;
use plc_ast::ast::*;
use xml::{attribute::Attribute, common::XmlVersion, name::Name, namespace::Namespace, writer::XmlEvent, EmitterConfig, EventWriter};

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
        let _ = parse_types(current_unit, unit_name, schema_path, &mut output_root);
        let _ = parse_pous(current_unit, unit_name, schema_path, &mut output_root);
    }
    write_xml_file(output_path, output_root)?;
    Ok(())
}

fn parse_types(current_unit: &CompilationUnit, unit_name: &str, schema_path: &'static str, output_root: &mut Node) -> Result<(), ()> {
    //Structs


    //Enums


    //Unions


    Ok(())
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
    for b in 0..current_unit.global_vars.len() {
        let current_global = &current_unit.global_vars[b];
        let mut parsed_variables: Vec<Box<dyn IntoNode>> = Vec::with_capacity(current_global.variables.len());
        
        for c in 0..current_global.variables.len() {
            let current_variable = &current_global.variables[c];

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

            println!("{}", &typename);

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
