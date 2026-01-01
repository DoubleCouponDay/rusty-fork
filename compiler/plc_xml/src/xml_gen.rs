use crate::serializer::*;

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
