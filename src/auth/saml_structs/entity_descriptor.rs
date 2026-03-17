use std::io::Cursor;

use chrono::{DateTime, SecondsFormat, Utc};
use quick_xml::{
    Writer,
    events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event},
};
use samael::{metadata::SpSsoDescriptor, signature::Signature};

use crate::AppError;

/// "md:EntityDescriptor" used for serializing the SP metadata XML
#[derive(Clone, Debug)]
pub struct EntityDescriptor {
    /// "@entityID"
    pub entity_id: String,
    /// "@ID"
    pub id: String,
    /// "Signature"
    pub signature: Signature,
    /// "@validUntil"
    pub valid_until: DateTime<Utc>,
    /// "SPSSODescriptor"
    pub sp_sso_descriptors: Vec<SpSsoDescriptor>,
}

const ENTITY_DESCRIPTOR_NAME: &str = "md:EntityDescriptor";

impl TryFrom<&EntityDescriptor> for Event<'_> {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: &EntityDescriptor) -> Result<Self, Self::Error> {
        let mut write_buf = Vec::new();
        let mut writer = Writer::new(Cursor::new(&mut write_buf));
        writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;

        let mut root = BytesStart::new(ENTITY_DESCRIPTOR_NAME);
        root.push_attribute(("xmlns:md", "urn:oasis:names:tc:SAML:2.0:metadata"));
        root.push_attribute(("xmlns:saml", "urn:oasis:names:tc:SAML:2.0:assertion"));
        root.push_attribute(("xmlns:mdrpi", "urn:oasis:names:tc:SAML:metadata:rpi"));
        root.push_attribute(("xmlns:mdattr", "urn:oasis:names:tc:SAML:metadata:attribute"));
        root.push_attribute(("xmlns:mdui", "urn:oasis:names:tc:SAML:metadata:ui"));
        root.push_attribute(("xmlns:ds", "http://www.w3.org/2000/09/xmldsig#"));
        root.push_attribute((
            "xmlns:idpdisc",
            "urn:oasis:names:tc:SAML:profiles:SSO:idp-discovery-protocol",
        ));

        root.push_attribute(("ID", value.id.as_str()));
        root.push_attribute(("entityID", value.entity_id.as_str()));
        root.push_attribute((
            "validUntil",
            value
                .valid_until
                .to_rfc3339_opts(SecondsFormat::Secs, true)
                .as_str(),
        ));

        writer.write_event(Event::Start(root))?;
        writer.write_event(TryInto::<Event<'_>>::try_into(&value.signature)?)?;
        for descriptor in &value.sp_sso_descriptors {
            writer.write_event(TryInto::<Event<'_>>::try_into(descriptor)?)?;
        }
        writer.write_event(Event::End(BytesEnd::new(ENTITY_DESCRIPTOR_NAME)))?;

        Ok(Event::Text(BytesText::from_escaped(String::from_utf8(
            write_buf,
        )?)))
    }
}

impl TryFrom<samael::metadata::EntityDescriptor> for EntityDescriptor {
    type Error = AppError;

    fn try_from(value: samael::metadata::EntityDescriptor) -> Result<Self, Self::Error> {
        Ok(EntityDescriptor {
            entity_id: value.entity_id.ok_or(AppError::InternalServerError)?,
            id: value.id.ok_or(AppError::InternalServerError)?,
            signature: value.signature.ok_or(AppError::InternalServerError)?,
            valid_until: value.valid_until.ok_or(AppError::InternalServerError)?,
            sp_sso_descriptors: value
                .sp_sso_descriptors
                .ok_or(AppError::InternalServerError)?,
        })
    }
}
