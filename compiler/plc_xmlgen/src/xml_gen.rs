use plc_ast::ast::CompilationUnit;

use super::serializer::*;

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
///     <ContentHeader name=\"Sample\">
///     </ContentHeader>
///     <Types>
///         <GlobalNamespace>
///         </GlobalNamespace>
///     </Types>
///     <Instances>
///     </Instances>
/// </Project>
pub fn get_omron_template() -> Node {
    let output = Node::new("Project")
        .attribute("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance")
        .attribute("xmlns:smcext", "https://www.ia.omron.com/Smc")
        .attribute("xsi:schemaLocation", "https://www.ia.omron.com/Smc IEC61131_10_Ed1_0_SmcExt1_0_Spc1_0.xsd")
        .attribute("schemaVersion", "1")
        .attribute("xmlns", "www.iec.ch/public/TC65SC65BWG7TF10")
            .child(&SFileHeader::new()
                .attribute("companyName", "OMRON Corporation")
                .attribute("productName", "Sysmac Studio")
                .attribute("productVersion", "1.30.0.0"))
            .child(&SContentHeader::new()
                .attribute("name", "Sample"))
            .child(&STypes::new()
                .child(&SGlobalNamespace::new()))
            .child(&SInstances::new());
        
    output
}

pub fn parse_project_into_nodetree(output: &Node, annotated_project: &Vec<&CompilationUnit>) {
    for a in 0..=annotated_project.len() {
        let current_unit = annotated_project[a];

        //global variables
        for b in 0..=current_unit.global_vars.len() {
            let current_global = &current_unit.global_vars[b];
            current_global.
        }
        
        //Structs


        //Functions


        //Enums


        //Unions


        //Function blocks


        //Programs
    }
}

pub fn write_xml_file() {

}
