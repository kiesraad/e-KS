//! Model H 3-2: authorisation to place a combined designation above a
//! candidate list.

use textris_pdf::build::{Textris, text};

use super::{
    h3::{
        DESIGNATION_SECTION, H3, authorisation_signature, candidates_section, districts_section,
        election_section, permission_section,
    },
    layout::{start_document, translator},
};

pub(super) fn document(input: &H3) -> Textris {
    let trans = translator(input.common.locale);
    let mut doc = start_document(
        "Model H 3-2",
        trans(
            "Machtiging om samengevoegde aanduiding boven kandidatenlijst te plaatsen",
            "Machtiging om gearfoege oantsjutting boppe kandidatelist te pleatsen",
        ),
        input.common.locale,
        Some((input.common.event_id, &input.common.sha_hash)),
    );
    doc.paragraph(trans(
        "Met dit formulier geeft u de inleveraar van de kandidatenlijst toestemming om een aanduiding boven de kandidatenlijst te plaatsen, die is gevormd door samenvoeging van de aanduidingen van politieke groeperingen of afkortingen daarvan.",
        "Mei dit formulier jouwe jo dejinge dy’t de kandidatelist ynleveret tastimming om in oantsjutting boppe de kandidatelist te pleatsen, dy’t foarme is troch gearfoeging fan de oantsjuttings fan politike groepearrings of ôfkoartings dêrfan.",
    ));
    doc.paragraph(trans(
        "U kunt alleen toestemming geven als u hiertoe gemachtigd bent door uw politieke groepering.",
        "Jo kinne allinnich tastimming jaan as jo dêrta machtige binne troch jo politike groepearring.",
    ));

    election_section(&mut doc, input);
    districts_section(&mut doc, input);

    doc.h3_numbered(trans(
        "Aanduiding van de politieke groeperingen",
        "Oantsjutting fan de politike groepearrings",
    ))
    .anchor(DESIGNATION_SECTION);
    doc.paragraph(
        text(trans(
            "De samengevoegde aanduiding van de politieke groeperingen: ",
            "De gearfoege oantsjutting fan de politike groepearrings: ",
        ))
        .bold(&input.common.designation),
    );

    permission_section(&mut doc, input, true);
    candidates_section(&mut doc, input);

    doc.h3_numbered(trans(
        "Ondertekening door de gemachtigden",
        "Undertekening troch de lêsthawwer",
    ));
    for (index, authorisation) in input.name_authorisations.iter().enumerate() {
        doc.h4(format!(
            "{} {}",
            trans(
                "Gemachtigde van politieke groepering",
                "Lêsthawwer fan politike groepearring",
            ),
            index + 1
        ));
        authorisation_signature(&mut doc, input.common.locale, authorisation);
    }

    doc
}
