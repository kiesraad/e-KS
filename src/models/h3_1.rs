//! Model H 3-1: authorisation to place a registered designation above a
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
        "Model H 3-1",
        trans(
            "Machtiging om aanduiding boven kandidatenlijst te plaatsen",
            "Machtiging om oantsjutting boppe kandidatelist te pleatsen",
        ),
        input.common.locale,
        Some((input.common.event_id, &input.common.sha_hash)),
    );
    doc.paragraph(trans(
        "Met dit formulier geeft u de inleveraar van de kandidatenlijst toestemming om de aanduiding die door uw politieke groepering is geregistreerd boven de kandidatenlijst te plaatsen.",
        "Mei dit formulier jouwe jo dejinge dy’t de kandidatelist ynleveret tastimming om de oantsjutting dy’t troch jo politike groepearring registrearre is boppe de kandidatelist te pleatsen.",
    ));
    doc.paragraph(trans(
        "U kunt alleen toestemming geven als u hiertoe gemachtigd bent door uw politieke groepering.",
        "Jo kinne allinnich tastimming jaan as jo dêrta machtige binne troch jo politike groepearring.",
    ));

    election_section(&mut doc, input);
    districts_section(&mut doc, input);

    doc.h3_numbered(trans(
        "Aanduiding van de politieke groepering",
        "Oantsjutting fan de politike groepearring",
    ))
    .anchor(DESIGNATION_SECTION);
    doc.paragraph(
        text(trans(
            "De geregistreerde aanduiding van de politieke groepering: ",
            "De registrearre oantsjutting fan de politike groepearring: ",
        ))
        .bold(&input.common.designation),
    );

    permission_section(&mut doc, input, false);
    candidates_section(&mut doc, input);

    doc.h3_numbered(trans(
        "Ondertekening door de gemachtigde van de politieke groepering",
        "Undertekening troch de lêsthawwer fan de politike groepearring",
    ));
    if let Some(authorisation) = input.name_authorisations.first() {
        authorisation_signature(&mut doc, input.common.locale, authorisation);
    }

    doc
}
