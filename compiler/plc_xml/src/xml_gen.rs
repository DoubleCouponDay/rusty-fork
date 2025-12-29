use crate::serializer::Node;



pub fn get_omron_template() -> &'static str {
    "<?xml version=\"1.0\"?>
        <Project xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xmlns:smcext=\"https://www.ia.omron.com/Smc\" xsi:schemaLocation=\"https://www.ia.omron.com/Smc IEC61131_10_Ed1_0_SmcExt1_0_Spc1_0.xsd\" schemaVersion=\"1\" xmlns=\"www.iec.ch/public/TC65SC65BWG7TF10\">
        <FileHeader companyName=\"OMRON Corporation\" productName=\"Sysmac Studio\" productVersion=\"1.30.0.0\" />
        <ContentHeader name=\"Sample\">
        </ContentHeader>
        <Types>
            <GlobalNamespace>
            </GlobalNamespace>
        </Types>
        <Instances>
        </Instances>
    </Project>"
}

