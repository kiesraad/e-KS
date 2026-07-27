Source: https://tvs.dictu.nl/sites/default/files/documents/Koppelvlakspecificatie-eID-SAML-v4.4.pdf

# eID SAML 4.4 Specification — Requirements

Source: *Koppelvlakspecificatie eID SAML v4.4 (16 september 2020, Definitief)*
Published by: Logius, Ministerie van Binnenlandse Zaken en Koninkrijksrelaties

---

## 1 Disclaimer

- This document is not a normative source for conducting audits on eID participants.
- The 4.4 version describes representation information exchange between DV/LC and RD, but this functionality will not be available initially.

## 2 History

| Version   | Changes |
|-----------|---------|
| 4.4 RC1   | Initial version |
| 4.4 RC2   | Corrections related to test findings |
| 4.4 RC3   | Added optional @ProviderName to AuthnRequest for eIDAS; support for multi-use certs in metadata; use of sender/receiver instead of specific roles |
| 4.4 final | Added requestorID in AuthnRequest for representation; fixed naming-scheme (base URN `urn:nl-eid-gdi:1.0`); fixed validUntil/cacheDuration; clarified certificate-use in signatures |

## 3 Frameworks

Based on OASIS SAML 2.0:
- saml-core-2.0-os
- saml-profiles-2.0-os
- saml-metadata-2.0-os
- saml-bindings-2.0-os
- saml errata

Also references:
- NORA (Nederlandse Overheid Referentie Architectuur)
- NCSC ICT-beveiligingsrichtlijnen voor TLS

### 3.1 SAML Profiles

Two profiles are used:
1. **Web Browser SSO profile** with HTTP-POST binding
2. **Single Logout profile** with HTTP-POST binding, issued by Session Participant to Identity Provider

### 3.1.1 SAML Message Flows and Bindings

#### Front-channel (re)authentication

| Step | Route | Message | Endpoint | Binding | Metadata |
|------|-------|---------|----------|---------|----------|
| 2 | DV/LC → Browser → RD | AuthnRequest | SingleSignOnService | HTTP-POST | RD IdP |
| 4 | RD → Browser → DV/LC | Artifact | AssertionConsumerService | HTTP-Artifact | DV/LC SP |

#### Back-channel (Assertion)

| Step | Route | Message | Endpoint | Binding | Metadata |
|------|-------|---------|----------|---------|----------|
| 5 | DV/LC → RD | ArtifactResolve | ArtifactResolutionService | SOAP | RD IdP |
| 6 | RD → DV/LC | ArtifactResponse | (direct response) | SOAP | — |

#### 3.1.1.1 SingleLogout Messages and Bindings

- Only SP-initiated logout is supported by the RD within an SSO federation context.
- IdP-initiated Logout is NOT supported.
- A DV participating in an SSO federation MUST send a Logout request to the RD (step 2a/b).
- The RD validates the LogoutRequest; if an active session exists for that user's browser, the RD terminates it.
- The RD replies with a LogoutResponse with success status if the LogoutRequest was valid.

| Step | Route | Message | Endpoint | Binding | Metadata |
|------|-------|---------|----------|---------|----------|
| 2 | DV/LC → Browser → RD | LogoutRequest | SingleLogoutService | HTTP-POST | RD IdP |
| 6 | RD → Browser → DV/LC | LogoutResponse | SingleLogoutService | HTTP-POST | DV/LC SP |

## 4 Glossary

| Term | English | Description |
|------|---------|-------------|
| Artifact | Artifact | Pointer to a SAML message sent through the front-channel to avoid exposing sensitive data to the end-user's UA |
| Assertion | Assertion | SAML Assertion |
| Back channel | Back channel | Communication channel between DV/LC, RD, AD, BVD, eTD (not interacting with end user) |
| LoA | Level of Assurance | Betrouwbaarheidsniveau |
| BVD | BVD | Bevoegdheidsverklaringsdienst |
| DV | SP | Dienstverlener (Service Provider) |
| SSO | Single Sign On | Eenmalig Inloggen |
| Front channel | Front channel | Communication between DV/LC, RD, AD or BVD and UA of End-user |
| Identity Provider (IDP) | Identity Provider (IDP) | De AuthentcatieDienst (AD) |
| LC | Cluster Connection Provider | Leverancier Clusteraansluiting |
| Metadata | Metadata | Before a SAML connection can be established, all parties must exchange connection properties through Metadata |
| Participant | Participant | Any party in authentication/representation processes (DV, RD, LC, AD, BVD) |
| RD | Routeringsdienst | See Roles |
| RV | Routeringsvoorziening | Facility which unburdens Service Providers when accepting multiple Identity Providers |
| SAML | SAML | SAML v 2.0 standard |
| SLO | SLO | Single Log Off |
| UA | UA | User Agent (e.g., browser) |

### 4.1 Roles

| Abbreviation | Role | Description | Example |
|-------------|------|-------------|---------|
| AD | Authenticatiedienst | Identity Provider (IDP) | DigiD, eTD AD, eIDAS out |
| DV | Dienstverlener | Service Provider (SP) | Gemeente, huisarts, overheidsinstelling |
| RD | Routeringsdienst | Technical realization of the Routeringsvoorziening | TVS, IdentityBridge |
| RV | Routeringsvoorziening | Facility which unburdens SPs when accepting multiple IDPs | Beheerorganisatie Routeringsvoorziening |
| LC | Leverancier Clusteraansluiting | Cluster connection provider; assists the DV in connecting to the RD | SaaS providers within health-care field |
| BVD | Bevoegdheidsverklaringsdienst | Service providing assertions for representation relationships | De BVD van programma Machtigen |
| MR | Mandate Register | Public or Private entity registering formalized representation relationships | eHerkenning MR's |

## 5 Introduction

### 5.1 Introduction

eID SAML 4.4 specifies the communication between Dienstverlener (DV) and Routeringsdienst (RD), and between Leverancier Clusteraansluiting (LC) and RD.

### 5.2 Interface Versioning

| Version | Description | Status |
|---------|-------------|--------|
| 4.4 | First version of eID SAML specs for connecting to an RD. Scope: DV-RD and LC-RD. | Final |
| 4.0 | DigiD CA 4.0 Specification. Introducing Encrypted BSN and support for LC. | Pilot |

## 6 Supported Use Cases (DV/LC - RD)

### 6.1 Authentication

- An End User authenticates on his/her own behalf at a DV which requires authentication.
- The DV redirects to the RD requesting authentication.
- At the RD, the End User selects an AD.
- After successful authentication at the AD, the End User is redirected back to the RD.
- The RD provides an interface response to the DV including at minimum: an identifier of the End User, the Level of Assurance, and the Service authenticated for.
- The DV can then take an access control decision.

#### 6.1.1 Actors
End User, DV, RD, AD

### 6.2 Authentication with Representation

- An Acting End User authenticates with the intent to consume a Service on behalf of another person.
- The representation relationship must be registered in a Machtigingenregister (MR).
- After successful authentication at the AD, the RD redirects to a BVD where the Acting End User selects the representation relationship.
- The RD includes both the attestation of identity and the attestation of representation in the response.

#### 6.2.1 Actors
Acting End User, Represented party, DV, RD, AD, BVD

### 6.3 Cluster Connection Connectivity

- In SaaS/multi-tenant solutions, the software vendor acts as an LC.
- The LC is registered with the RD and registers all DV's it provides access for.
- DV initiates authentication via the LC. The LC sends the AuthnRequest to the RD for the DV.
- Data in the response from the RD is encrypted to the DV, not the LC.
- The LC facilitates the authentication process but cannot access the sensitive information.

**Note:** The Cluster Connection is applicable to SaaS and PaaS (if eID-connectivity is direct), but NOT to IaaS.

#### 6.3.1 Actors
DV, LC, RD

### 6.4 Authentication with AD/BVD Preselection

- The User makes the AD selection at the DV rather than at the RD, improving UX.
- The DV includes the preselection in the AuthnRequest via the Scoping/IDPList element.
- The RD applies the preselection as a filter on available options.

**Requirement:** Service Providers SHOULD offer the choice for each AD/BVD in a non-discriminatory way for all applicable AD/BVDs.

#### 6.4.1 Actors
End-User, DV, RD, AD, BVD

## 7 SAML Message Specification

### 7.1 SAML Authentication Steps

1. End user (UA) wants to access a part of the web service requiring authentication.
2. The DV or LC sends the end-user to the RD for authentication and/or proof of representation.
3. (Out of scope) The RD offers the user a choice of AD's and BVD's meeting the authentication requirements.
4. The RD sends the end user back to the DV or LC via a redirect. A meaningless **artifact** is sent (not the actual response). Even if authentication was unsuccessful, an artifact is sent.
5. The DV or LC retrieves the response message from RD via the **back channel** based on the artifact. Artifacts are stored by RD for a maximum of **15 minutes** and can only be retrieved **once**.
6. RD replies with the ArtifactResponse containing the Response and Assertion. If successful, the requested identities and attributes are encrypted so that only the intended DV(s) can obtain plain text.
7. & 8. Successful authentication provides the DV the information needed for access control.

**Important:** The distinction between front channel and back channel ensures that user-defined attributes (e.g., BSN) are never sent via the front channel, preventing interception by the browser.

### 7.2 SAML Message Specification

Rules:
1. **Messages contain at least the elements specified as mandatory by the standard.**
2. **Messages also contain optional elements. It is indicated whether these are mandatory, conditional, or optional.**
3. **Optional SAML elements that are not in this specification SHOULD NOT be included. When present they will be ignored when possible.**

### 7.3 SAML AuthnRequest

Sender: DV or LC
Recipient: RD

| Element/@Attribute | Card. | Description |
|-------------------|-------|-------------|
| @ID | 1 | Unique message identifier. MUST identify the message uniquely within the scope of sender and receiver for at least 12 months. |
| @Version | 1 | MUST be '2.0'. |
| @IssueInstant | 1 | Time of issuing the request. |
| @Destination | 1 | URL of the recipient. MUST match the metadata. |
| @ForceAuthn | 0..1 | 'true' indicates existing SSO session MUST NOT be used. Default is 'false'. |
| @AssertionConsumerServiceIndex | 1 | This index MUST refer to an endpoint of an AssertionConsumerService in the issuer's metadata. Note: @AssertionConsumerServiceURL MUST NOT be included. |
| @ProviderName | 0..1 | Conditional. Reserved for eIDAS UIT. SHOULD NOT be used in other use cases. |
| Issuer | 1 | MUST contain the EntityID of the issuer as registered in the metadata. |
| Signature | 1 | MUST contain the XML signature of the sender for the enveloped message. MUST contain a `<KeyInfo>` element with a `<KeyName>` or `<X509Certificate>`. |
| AttributeConsumingServiceIndex | 0..1 | Conditional. Only one of `<Extensions>` or `<AttributeConsumingServiceIndex>` MUST be present. MAY only be used if the issuer is a DV. MUST NOT be used in other cases. If present, MUST refer to an AttributeConsumingService in the DV's metadata. |
| Extensions | 0..1 | Conditional. Only one of `<Extensions>` or `<AttributeConsumingServiceIndex>` MUST be present. MUST be included if the issuer is not a DV. MAY be used when the issuer is a DV. |
| -Attribute (IntendedAudience) | 1 | An `<Attribute>` with @Name="urn:nl-eid-gdi:1.0:IntendedAudience" MUST be present and contain an AttributeValue with the EntityID of the DV for which authentication is requested. |
| -Attribute (ServiceUUID) | 1 | An `<Attribute>` with @Name="urn:nl-eid-gdi:1.0:ServiceUUID" MUST be present and contain an AttributeValue with a ServiceUUID known at the service catalogus of the receiver. |
| Scoping | 0..1 | OPTIONAL element. |
| -IDPList | 0..1 | OPTIONAL. MAY be used to limit the AD/BVD selection at the RD. |
| --IDPEntry | 1..n | At least one IDPEntry MUST be present if IDPList is present. If no valid IDPEntry is present, the AuthnRequest will fail with `urn:oasis:names:tc:saml:2.0:status:Requester` and the applicable second-level code. MUST contain at least the EntityID of one AD. If it also contains a BVD EntityID, representation using the BVD is optional. |
| ---@ProviderID | 1 | MUST contain the EntityID of a pre-selected AD or BVD. |
| -RequesterID | 0..n | Optional. MAY be used to make representation with a BVD mandatory. If used, MUST contain one or more EntityID's of BVD's. |

#### 7.3.1.1 Use of `<AttributeConsumingService>` or `<Extensions>` for Service Definitions

- A service definition has a unique UUID (ServiceUUID) in the service catalog.
- Only a DV MAY use the AttributeConsumingServiceIndex. All other participants MUST use the `<Extensions>` element.
- A DV MAY use either the `<AttributeConsumingServiceIndex>` or the `<Extensions>` element.
- When using the AttributeConsumingService, the RD will retrieve the DV's EntityID from the `<Issuer>` element.

#### 7.3.1.2 Processing Rules

The RD MUST:
1. Validate that the DV is registered for the requested ServiceDefinition (serviceUUID) referenced by the ServiceIndex in the DV metadata or the `<Extensions>`, and that the registration is valid.
2. (When an LC is involved) validate that the DV is registered to use the requested ServiceUUID with the LC that sends the AuthnRequest on behalf of the DV, and that the registration is valid.
3. If any of these validations fails, the authentication MUST fail.

### 7.4 SAML AuthnRequest Response Message

- The receiver of the AuthnRequest message sends a SAML-artifact via the front channel by a redirect to the AssertionConsumerService referenced in the AuthnRequest.
- An artifact is a reference to the SAML Response message.
- Even if no authentication has taken place, an artifact will be sent.
- The artifact is sent to the recipient via an HTTP Redirect.

### 7.5 SAML ArtifactResolve

Sent via **SOAP binding** over the **back channel** protected with **two-sided TLS authentication**.

Sender: DV or LC
Recipient: RD

| Element/@Attribute | Card. | Description |
|-------------------|-------|-------------|
| @ID | 1 | Unique message identifier. MUST identify the message uniquely for at least 12 months. |
| @Version | 1 | MUST be '2.0'. |
| @IssueInstant | 1 | Time at which the message was created. |
| @Destination | 0..1 | MAY be included. If included, MUST contain URL of the receiver matching one of the `<ArtifactResolutionService>` elements in the receiver's metadata. |
| Issuer | 1 | MUST contain the EntityID of the sender. |
| Signature | 1 | MUST contain the Digital signature of the sender. MUST contain a `<KeyInfo>` with `<KeyName>` or `<X509Certificate>`. |
| Artifact | 1 | Contains the Artifact that was received as query parameter. |

### 7.6 SAML ArtifactResponse

The `<ArtifactResponse>` is the response to the `<ArtifactResolve>` request in a SOAP message. It in turn contains the `<Response>` to the Original AuthnRequest.

Sender: RD
Recipient: DV or LC

#### 7.6.1 `<ArtifactResponse>`

| Element/@Attribute | Card. | Description |
|-------------------|-------|-------------|
| @ID | 1 | Unique message identifier (at least 12 months). |
| @InResponseTo | 1 | Unique @ID attribute of the ArtifactResolve request. |
| @Version | 1 | MUST be '2.0'. |
| @IssueInstant | 1 | Time of issuing the Response. |
| Issuer | 1 | MUST contain the entityID of the sender. |
| Signature | 1 | MUST contain the Digital signature of the sender. MUST contain a `<KeyInfo>` element with a `<KeyName>` element. |
| Status | 1 | MUST contain a `<StatusCode>` element with the status of the artifact resolve. |
| -StatusCode @Value | 1 | Top-level list per SAML core section 3.2.2.2, following SAML-bindings section 3.6.6. |
| --StatusCode | 0..1 | Conditional. Should only be present if top-level StatusCode is not 'Success'. |
| -StatusMessage | 0..1 | Only present if top-level StatusCode is not 'Success'. MAY contain a message detailing the error. |
| Response | 0..1 | Conditional. If the artifact resolves to a response, this MUST contain the `<Response>` to the AuthnRequest. |

**Special `<Status>` rules per SAML-bindings section 3.6.6:** Even if the ArtifactResponse's Status indicates "Success", it may still not contain a Response if the artifact requester is not authorized or the artifact is no longer valid.

#### 7.6.2 `<Response>`

| Element/@Attribute | Card. | Description |
|-------------------|-------|-------------|
| @ID | 1 | Unique message characteristic (at least 12 months). |
| @InResponseTo | 1 | Unique @ID attribute of the AuthnRequest. |
| @Version | 1 | MUST be '2.0'. |
| @IssueInstant | 1 | Time of issuing the Response. |
| @Destination | 1 | URL of the endpoint. MUST match a recipient's metadata AssertionConsumerService. |
| Issuer | 1 | MUST contain the EntityID of the sender. |
| Signature | 0..1 | SHOULD NOT be used as the `<Response>` is part of the `<ArtifactResponse>` which is already signed by the RD. If included, MUST contain a `<KeyInfo>` element with a `<KeyName>` or `<X509Certificate>`. |
| Status | 1 | MUST contain a `<StatusCode>` element with the status of the authentication. |
| -StatusCode @Value | 1 | If not 'Success', additional info SHOULD be in the embedded StatusCode element. |
| --StatusCode | 0..1 | Conditional. Should only be present if top-level is not 'Success'. |
| --@Value | 1 | In the event of a cancellation or error, MUST be populated with "AuthnFailed". |
| -StatusMessage | 0..1 | Only present if top-level StatusCode is not 'Success'. MUST contain exact phrase 'Authentication cancelled' when authentication is cancelled. |
| Assertion | 0..1 | Conditional. MUST be present if status is "Success". MUST NOT be included otherwise. |
| EncryptedAssertion | 0 | **MUST NOT be included.** |

#### 7.6.3 SAML Assertion

Issuer = RD
Recipient = DV or LC

| Element/@Attribute | Card. | Description |
|-------------------|-------|-------------|
| @ID | 1 | Unique message identifier (at least 12 months). |
| @Version | 1 | MUST be '2.0'. |
| @IssueInstant | 1 | Time of issuance of the assertion. |
| Issuer | 1 | MUST contain the EntityID of the issuer. |
| Signature | 1 | MUST contain the Digital signature of the sender. MUST contain a `<KeyInfo>` element with a `<KeyName>` element. |
| Subject | 1 | MUST be included. |
| -NameID | 1 | NameID MUST contain a TransientID. |
| -SubjectConfirmation | 1 | Contains the `<SubjectConfirmation>` conform the WebSSO profile. |
| Conditions | 1 | NotBefore and NotOnOrAfter limit the window during which the assertion can be delivered. |
| -@NotBefore | 1 | MUST be included. |
| -@NotOnOrAfter | 1 | MUST be included. |
| -AudienceRestriction | 1 | MUST be included. |
| --Audience | 1..n | MUST contain the EntityID(s) of all parties intended to receive and process the assertion. MUST always contain the DV's EntityID. If an LC is involved, MUST also contain the LC's EntityID. |
| AuthnStatement | 1 | MUST be included. |
| -@AuthnInstant | 1 | MUST contain the time of creation of the enclosing Assertion. |
| -AuthnContext | 1 | MUST be included. |
| --AuthnContextClassRef | 1 | MUST contain the level of assurance at which authentication took place. |
| --AuthenticatingAuthority | 0..n | MUST contain the EntityID(s) of all authorities involved in the authentication and representation assertion issuance except for the assertion issuer. |
| AttributeStatement | 0..1 | Conditional. MUST be included if StatusCode is 'Success'. MUST NOT be included otherwise. |
| Advice | 1 | MUST be included. Contains the original assertions received from AD and BVD. |
| -Assertion | 1..n | Contains the original `<Assertion>` elements. MUST contain the original AD `<Assertion>`. MAY contain the original BVD `<Assertion>` in case of representation. |

##### 7.6.3.1 Audience Restriction

- An Assertion may only be processed if the `<AudienceRestriction>` contains the `<EntityID>` of the recipient.
- The values for DVs included in `<Audience>` are reflected in the @Recipient attribute of the `<EncryptedID>` elements in the `<AttributeStatement>`.
- The LC is not a `<Recipient>` and cannot decrypt EncryptedID's or attributes.

##### 7.6.3.2 Level of Assurance Validation

- The `<AuthnContextClassRef>` always states the authentication level at which the citizen authenticated.
- DV's MUST be prepared for a higher LoA than requested.
- DV's MUST accept authentications with a level equal to or higher than the minimum level registered for the Service.
- DV must configure minimum LoA for the Service when providing information in the onboarding process.
- The AD is responsible for providing the correct LoA for a given authentication-request.

##### 7.6.3.3 SubjectConfirmation

| Element/@Attribute | Card. | Description |
|-------------------|-------|-------------|
| SubjectConfirmation | 1 | Association of client with assertion to conform to SAML Web SSO profile. |
| -@Method | 1 | MUST contain "urn:oasis:names:tc:SAML:2.0:cm:bearer". |
| -SubjectConfirmationData | 1 | MUST be included. |
| --@NotOnOrAfter | 1 | Initially set to +2 minutes relative to creation time. The @NotBefore MUST NOT be used. |
| --@Recipient | 1 | The assertion consumer service URL of the immediate requester. |
| --@InResponseTo | 1 | The @ID of the `<AuthnRequest>` this Assertion is in response to. A receiving DV or LC MUST verify this value corresponds to the initiating AuthnRequest @ID. |

##### 7.6.3.4 AttributeStatement

- When present, MUST contain an `<AttributeStatement>`.
- `<AttributeStatement>` contains one or more `<Attribute>` elements.
- MUST contain at least one `<Attribute>` with @Name="urn:nl-eid-gdi:1.0:ActingSubjectID".
- MAY contain an `<Attribute>` with @Name="urn:nl-eid-gdi:1.0:LegalSubjectID" indicating representation.

**Attribute structure:**

| Element/@Attribute | Card. | Description |
|-------------------|-------|-------------|
| AttributeStatement | 1 | MUST be included. |
| -Attribute | 1..n | See tables below. |
| --@Name | 1 | MUST contain the type of the attribute. |
| --AttributeValue | 1..n | MUST contain one or more AttributeValues — one for each recipient. |
| ---EncryptedID | 1 | MUST contain one encrypted `<NameID>` element. |
| ----EncryptedData | 1 | MUST contain the encrypted data containing the XML encrypted NameID (BSN). |
| ----EncryptedKey | 1..n | MUST contain the wrapped decryption keys. This element MUST include the intended Recipient. |
| -----@Recipient | 1 | The recipient (DV, LC or RD) for which this EncryptedID is intended. MUST contain an EntityID. |

**Unencrypted attributes (always present):**

| Attribute | Card. | @Name | Description |
|-----------|-------|-------|-------------|
| ServiceUUID | 1 | urn:nl-eid-gdi:1.0:ServiceUUID | The ServiceUUID for which this Assertion is intended. |

**Encrypted attributes — Authentication:**

| Attribute | Card. | @Name | Description |
|-----------|-------|-------|-------------|
| ActingSubjectID | 1 | urn:nl-eid-gdi:1.0:ActingSubjectID | Contains the identity of the authenticated subject. |

**Encrypted attributes — Representation:**

| Attribute | Card. | @Name | Description |
|-----------|-------|-------|-------------|
| ActingSubjectID | 1 | urn:nl-eid-gdi:1.0:ActingSubjectID | The encrypted ActingSubjectID as received from the AD. |
| LegalSubjectID | 1..n | urn:nl-eid-gdi:1.0:LegalSubjectID | SAML eID 4.4 will only support 1 LegalSubjectID. The encrypted LegalSubjectID as received from BVD. |

**Multiple recipients:**

- Each EncryptedKey MUST have a CarriedKeyName equal to the KeyName used in the KeyInfo of the EncryptedData.
- Each EncryptedKey SHOULD have a ReferenceList referring back to the data encrypted with the symmetric key.
- Elements without an EncryptedKey intended for the decrypting recipient MAY be ignored.
- EncryptedKeys for other recipients of encrypted elements SHOULD be ignored.

##### 7.6.3.4.4 EncryptedID

- Identifiers (NameID) are contained in SAML `<EncryptedID>` elements in all cases.
- The specific type of identifier is communicated through a @NameQualifier attribute within the `<NameID>`.
- All identifiers are XML encrypted so that only the intended recipient(s) can decrypt.
- The intended recipient is communicated through the @Recipient attribute within the EncryptedKey element.

An `<EncryptedID>` MUST contain a SAML `<NameID>` after decryption, with the following properties:
- The Format attribute MUST be set to 'urn:oasis:names:tc:SAML:2.0:nameid-format:persistent'.
- The @NameQualifier attribute MUST be populated with the full name of the type of identifying attribute.
- The attributes SPNameQualifier and SPProvidedID MUST NOT be used.
- If more than one certificate is listed for encryption for the recipient in the metadata, the content-encryption-key MUST be encrypted for each certificate. This results in multiple `<EncryptedKey>` each with the same @Recipient.

##### 7.6.3.5 Response Message Processing Rules for DV

The service provider MUST do the following:
1. Verify any signatures present on the assertion(s) or the response.
2. Verify that the Recipient attribute in any bearer `<SubjectConfirmationData>` matches the assertion consumer service URL to which the `<Response>` or artifact was delivered.
3. Verify that the @NotOnOrAfter attribute in any bearer `<SubjectConfirmationData>` has not passed, subject to allowable clock skew.
4. Verify that the @InResponseTo attribute in the bearer `<SubjectConfirmationData>` equals the ID of its original `<AuthnRequest>` message.
5. Verify that it's EntityID is included as `<Audience>` in the `<Assertion>`.
6. Verify that any assertions relied upon are valid in other respects.
7. Any assertion which is not valid, or whose subject confirmation requirements cannot be met SHOULD be discarded and SHOULD NOT be used to establish a security context.

### 7.7 Federated Login and Logout

**SSO is defined at the AD level.** There is no SSO over AD's unless the AD's mutually agree.

A DV who wants to grant access through SSO can do so via the SSO service from the AD if the AD offers SSO. Details on the SSO service are to be provided by the AD.

Cases where the user is still asked to re-authenticate:
1. The LoA required by the service provider is higher than the level in the existing SSO session.
2. The existing SSO session applies to a different SSO federation.
3. The service provider includes `<ForceAuthn>` element with value True.
4. The existing SSO session has expired.

SP initiated logout is limited: sessions with other active DV's within the same federation will continue to be active until the local DV session times out or the user logs out.

#### 7.7.1 SP Initiated `<LogoutRequest>`

Sender: DV or LC
Recipient: RD

| Element/@Attribute | Card. | Description |
|-------------------|-------|-------------|
| @ID | 1 | Unique message attribute. |
| @Version | 1 | MUST be '2.0'. |
| @IssueInstant | 1 | Time at which the message was created. |
| @Destination | 1 | URL of the recipient. |
| Signature | 1 | MUST contain the Digital signature of the DV or LC. MUST contain a `<KeyInfo>` with `<KeyName>` or `<X509Certificate>`. When the sender is an RD, it MUST contain a `<KeyInfo>` with a `<KeyName>`. |
| NameID | 1 | MUST contain the TransientID `<NameID>` element from the `<Subject>` of the original Assertion. |
| Issuer | 1 | MUST contain the EntityID of the sender. |

#### 7.7.2 IdP `<LogoutResponse>`

Sender: RD
Recipient: DV or LC

| Element/@Attribute | Card. | Description |
|-------------------|-------|-------------|
| @ID | 1 | Unique message attribute. |
| @Version | 1 | MUST be '2.0'. |
| @IssueInstant | 1 | Time at which the message was created. |
| @Destination | 1 | URL of the recipient. |
| @InResponseTo | 1 | @ID of the LogoutRequest. |
| Signature | 1 | MUST contain the Digital signature. MUST contain a `<KeyInfo>` with `<KeyName>` or `<X509Certificate>`. When the sender is an RD it MUST contain a `<KeyInfo>` with a `<KeyName>`. |
| Issuer | 1 | MUST contain the EntityID of the sender. |
| Status | 1 | MUST contain a StatusCode element with the status of the logout. |

### 7.8 Error Codes

#### 7.8.1 Top-level Code

Standard SAML 2.0 error codes:

| Status Code | Description |
|------------|-------------|
| urn:oasis:names:tc:SAML:2.0:status:Requester | Errors caused by the initiator of the SAML request (e.g., unsupported assurance level, expired request). |
| urn:oasis:names:tc:SAML:2.0:status:Responder | Errors caused by the recipient (e.g., technical failure, unsupported functionality). |

#### 7.8.2 Second-level Status Codes

| Status Code | Description |
|------------|-------------|
| urn:oasis:names:tc:SAML:2.0:status:AuthnFailed | User cannot be authenticated (e.g., invalid credentials, cancel button used). |
| urn:oasis:names:tc:SAML:2.0:status:NoAuthnContext | User cannot be authenticated at the minimum level specified in the dienstencatalogus (DC). |
| urn:oasis:names:tc:SAML:2.0:status:RequestUnsupported | Message is correctly formatted and understood, but the requested functionality is not supported. |
| urn:oasis:names:tc:SAML:2.0:status:RequestDenied | SAML responder refuses to perform a message exchange (e.g., mandatory signature could not be verified). |
| urn:oasis:names:tc:SAML:2.0:status:NoSupportedIDP | None of the identity providers in an `<IDPList>` are supported by the intermediary. |

#### 7.8.3 Cancelling

- If a user cancels, the participant MUST direct the user to the latest sender of a SAML request, with valid SAML status codes: `urn:oasis:names:tc:SAML:2.0:status:Responder` with `urn:oasis:names:tc:SAML:2.0:status:AuthnFailed`.
- A `<StatusMessage>` element MUST be included, containing the exact phrase **"Authentication cancelled"**.
- If the RD receives a cancellation from an AD or BVD, it MUST ask the user to re-select or cancel (only if selection took place at the RD). Otherwise, the cancellation must be forwarded to the DV or LC.
- If the DV or LC receives a cancellation, it MUST indicate to the user that he is not logged in, and MAY offer re-authentication.

#### 7.8.4 Attributes Not Supported

- A participant receiving such a message MUST show the user a message indicating something went wrong (without revealing security sensitive details).
- MAY offer the user the option to cancel.

#### 7.8.5 Incorrect Message (Recoverable)

- The recipient MUST direct the user to the initiator of the SAML request, with status codes: `urn:oasis:names:tc:SAML:2.0:status:Responder` with `urn:oasis:names:tc:SAML:2.0:status:RequestUnsupported`.
- A `<StatusMessage>` element MUST be included, containing a description of the problem (e.g., "Level of assurance not supported").
- If the participant is an RD and no interaction with the user took place, the message must be forwarded to the DV or LC.

#### 7.8.6 Incorrect Message (Non-recoverable)

Examples: not a valid SAML message, XML does not match XSD, unknown issuer, invalid signature, invalid ServiceID/attributes/EntityConcernedTypes, response not matching the request.

- A participant MUST investigate the nature of the error.
- MUST show the user a message indicating a non-recoverable error.
- MUST return a SAML response with status codes: `urn:oasis:names:tc:SAML:2.0:status:Requester` and `urn:oasis:names:tc:SAML:2.0:status:RequestUnsupported`, or an HTTP error in case a synchronous response is expected (like SOAP).

## 8 SAML Metadata

### 8.1 TLS Certificates in Metadata

- TLS certificates MUST be included in the LC metadata as a signing certificate (@use=signing).
- Difference with normal signing certificates can be made via extended key usage (per SAML Version 2.0 Errata 05, E62).

### 8.2 General Processing Requirements

- The metadata MUST be validated by the consuming parties.
- The consuming parties MUST NOT use the metadata if the validation is not successful.

### 8.3 DV Metadata

Published by: DV connecting directly to RD
Consumed by: RD

| Element/@Attribute | Card. | Description |
|-------------------|-------|-------------|
| EntityDescriptor | 1 | MUST contain one `<EntityDescriptor>` with one `<SPSSODescriptor>`. |
| -@ID | 1 | Document-unique identifier, used as reference point when signing. |
| -@entityID | 1 | The unique identifier of the SAML entity. Contains the EntityID of the DV. |
| -@validUntil | 0..1 | MAY contain a datetime at which the metadata expires. Either validUntil or cacheDuration MUST be present. |
| -@cacheDuration | 0..1 | MAY contain cacheDuration. Either validUntil or cacheDuration MUST be present. |
| -Signature | 1 | Digital signature of the DV. MUST contain a `<KeyInfo>` element with a `<KeyName>` or `<X509Certificate>`. |
| -SPSSODescriptor | 1 | |
| --@AuthnRequestsSigned | 1 | MUST be set to "true". |
| --@protocolSupportEnumeration | 1 | MUST be set to "urn:oasis:names:tc:SAML:2.0:protocol". |
| --@WantAssertionsSigned | 1 | MUST be set to "true". |
| --KeyDescriptor | 2..n | MUST contain KeyDescriptor element(s) for signing and TLS. Can be achieved with 2 KeyDescriptor elements with @use="signing" or a single certificate supporting both functions. A second `<KeyDescriptor>` MAY be present for both to support certificate rollover. MUST contain at least 1 KeyDescriptor with @use="encryption". A second @use="encryption" MAY be present for rollover. All certificates must be PKIoverheid certificates. |
| ---KeyInfo | 1 | |
| ----KeyName | 1 | Contains the name which identifies the key. |
| ----X509Data | 1 | Contains the encoded PKIoverheid X509 certificate with the public key. |
| --SingleLogoutService | 0..n | Conditional: MUST be present if the DV supports SSO. At least one MUST contain HTTP-POST binding. |
| ---@Binding | 1 | MUST contain the appropriate binding for the endpoint. |
| ---@Location | 1 | MUST contain the URL of the SingleLogoutService endpoint. |
| --AssertionConsumerService | 1..n | Must contain at least one URL to redirect to after authentication. If more than one, one must have @isDefault="true". |
| ---@Binding | 1 | The binding. |
| ---@Location | 1 | The URL. |
| ---@Index | 1 | MUST be present. |
| ---@isDefault | 0..1 | MUST be present if more than one ACS. |
| --AttributeConsumingService | 0..n | Conditional: MUST be used if the DV does not support Extensions in the AuthnRequest. |
| ---@Index | 1 | MUST be present. |
| ---@isDefault | 0..1 | MUST be present if more than one index. |
| ---ServiceName | 1..n | One or more language-qualified names for the service. Only one per language. |
| ---RequestedAttribute | 1..n | At least one with @name="urn:nl-eid-gdi:1.0:ServiceUUID". |
| ----AttributeValue | 1 | MUST contain the ServiceUUID to be used. Must be pre-registered with the RV service catalogue (DC). |

### 8.4 LC SAML SP Metadata

Published by: LC
Consumed by: RD

#### 8.4.1 LC SP Metadata

Uses `<EntitiesDescriptor>` (plural) wrapping the LC's own EntityDescriptor plus one EntityDescriptor per DV.

| Element/@Attribute | Card. | Description |
|-------------------|-------|-------------|
| EntitiesDescriptor | 1 | Required element containing multiple EntityDescriptors. |
| -@ID | 1 | Document-unique identifier for signing. |
| -@validUntil | 0..1 | Either validUntil or cacheDuration MUST be present. |
| -@cacheDuration | 0..1 | Either validUntil or cacheDuration MUST be present. |
| -Signature | 1 | Digital signature of the LC. MUST contain a `<KeyInfo>` with `<KeyName>` or `<X509Certificate>`. |
| -EntityDescriptor | 1..n | MUST contain the LC's EntityDescriptor and EntityDescriptors of all DV's the LC supports. |

#### 8.4.2 LC EntityDescriptor within LC Metadata

Similar to DV metadata but with LC-specific differences:
- SPSSODescriptor with AuthnRequestsSigned="true", WantAssertionsSigned="true".
- 1..4 KeyDescriptor elements for signing and TLS, with optional rollover keys.
- TLS certificates for client authentication MUST be included as a signing certificate in the LC SAML metadata.
- All certificates must be PKIoverheid certificates containing the OIN of the EntityDescriptor's entityID.
- AssertionConsumerService with HTTP-Artifact binding.

#### 8.4.3 DV EntityDescriptor within LC Metadata

| Element/@Attribute | Card. | Description |
|-------------------|-------|-------------|
| EntityDescriptor | 1 | Per DV supported by the LC. |
| -@entityID | 1 | MUST contain the EntityID of the DV. |
| -SPSSODescriptor | 1 | protocolSupportEnumeration = SAML 2.0 protocol. |
| --KeyDescriptor | 1..2 | MUST contain at least 1 with @use="encryption". A second MAY be present for rollover. All certs must be PKIoverheid certs containing the OIN referred to in the entityID. |
| --AssertionConsumerService | 1..n | MUST contain only one entry. MUST contain a copy of the LC's ACS. Will be ignored as the LC's ACS definitions are used. |

### 8.5 RD SAML IdP Metadata

Published by: RD
Consumed by: DV and LC

| Element/@Attribute | Card. | Description |
|-------------------|-------|-------------|
| EntityDescriptor | 1 | |
| -@ID | 1 | Document-unique identifier for signing. |
| -@entityID | 1 | Contains the EntityID of the RD. |
| -@validUntil | 0..1 | Either validUntil or cacheDuration MUST be present. |
| -@cacheDuration | 0..1 | Either validUntil or cacheDuration MUST be present. |
| -Signature | 1 | Digital signature of RD. MUST contain a `<KeyInfo>` with a `<KeyName>` element. |
| -IDPSSODescriptor | 1 | |
| --@protocolSupportEnumeration | 1 | Set to "urn:oasis:names:tc:SAML:2.0:protocol". |
| --@WantAuthnRequestsSigned | 1 | Set to "true". |
| --KeyDescriptor | 1..n | At least 1 with @use="signing". |
| ---KeyInfo | 1 | |
| ----KeyName | 1 | Contains the name identifying the key. |
| ----X509Data | 1 | Contains the encoded X509 certificate. |
| --ArtifactResolutionService | 1..n | MUST be implemented at least once per service. |
| ---@Binding | 1 | SAML-SOAP binding only. |
| ---@Location | 1 | URL of the SAML artifact resolution endpoint. |
| ---@Index | 1 | MUST be unique for all ArtifactResolutionService elements. |
| --SingleSignOnService | 1..n | Endpoints supporting the Authentication Request protocol. |
| ---@Binding | 1 | HTTP-POST binding only. |
| ---@Location | 1 | URL of the SingleSignOnService endpoint. |
| --SingleLogoutService | 1..n | Endpoint for logout. |
| ---@Binding | 1 | MUST be set to "urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST". Other bindings are NOT supported. |
| ---@Location | 1 | The URL of the SAML endpoint. |

## 9 Technical Requirements and Recommendations

### 9.1 Signing, Encryption Algorithms and Hash Functions

eID SAML 4.x **no longer supports SHA1** except for the padding function (xmldsig # rsa-sha1). Only RSA is supported.

**Signing algorithms:**

| Algorithm | Namespace |
|-----------|-----------|
| RSAwithSHA256 | http://www.w3.org/2001/04/xmldsig-more#rsa-sha256 |
| RSAwithSHA384 | http://www.w3.org/2001/04/xmldsig-more#rsa-sha384 |
| RSAwithSHA512 | http://www.w3.org/2001/04/xmldsig-more#rsa-sha512 |

**Digest algorithms (minimum SHA256):**

| Algorithm | Namespace |
|-----------|-----------|
| SHA256 | http://www.w3.org/2001/04/xmlenc#sha256 |
| SHA384 | http://www.w3.org/2001/04/xmldsig-more#sha384 |
| SHA512 | http://www.w3.org/2001/04/xmldsig-more#sha512 |

**Signature requirements:**
- The digital signature is embedded in the message content with **Enveloped Signature Transform**.
- Canonicalization MUST be carried out according to the **exclusive c14n** method without comments (`http://www.w3.org/2001/10/xml-exc-c14n#`).
- Digests MUST be calculated with at minimum the SHA256 algorithm.
- The SignatureValue MUST be calculated with at minimum the RSAwithSHA256 algorithm.
- Participants MUST sign messages and metadata with a **PKIoverheid certificate** with a key length of at least **2048 bits**, containing the OIN (organisatie-identificatienummer) of the participant. The extended key usage MUST allow use for signing.
- The Reference MUST refer to the signed element via an ID attribute in the local document.

### 9.2 Signature

- Each `<Signature>` in SAML messages generated by a DV or LC and in the DV or LC SAML SP metadata MUST contain either a `<KeyInfo>` element with a `<X509Certificate>` element OR a `<KeyName>` element. The use of `<KeyName>` is preferred as this limits data amount.
- Each `<Signature>` in SAML messages generated by a RD and in the RD SAML IdP metadata **MUST** contain a `<KeyInfo>` element with a `<KeyName>` element containing a keyname that corresponds to a `<KeyName>` in a `<KeyDescriptor>` in the RD's metadata.
- Certificates used to verify a `<Signature>` MUST be retrieved from the party's **verified metadata**. The `<X509Certificate>` or `<KeyName>` in the `<KeyInfo>` of the Signature MUST only be used to retrieve the corresponding certificate from the verified metadata.

### 9.3 Encryption

- Encryption is achieved via XML-encryption.
- Block encryption algorithm: **AES-256** (`http://www.w3.org/2001/04/xmlenc#aes256-cbc`).
- Asymmetric encryption for key wrapping: RSA algorithm with **OAEP padding** and a **SHA digest** (`http://www.w3.org/TR/xmlenc-core1/#sec-RSA-OAEP`).
- The SHA1 version SHOULD NOT be used (`http://www.w3.org/2009/xmlenc11#mgf1sha1`).

### 9.4 TLS Transport

- The RD requires that a service provider always protects http traffic with **TLS v1.2 or higher** in accordance with the NCSC directive with 'good' assessment.
- The certificate must be issued under PKIoverheid with a key length of at least **2048 bits**.
- When connecting directly between the RD and the LC or DV (back channel), both parties must use a PKIoverheid certificate and **mutual authentication** is mandatory (**mutual TLS**).

### 9.5 NotBefore and NotOnOrAfter

- LCs and DVs must respect the NotBefore and NotOnOrAfter parameters and reject messages that do not comply.
- With a re-authentication, the entire protocol handling must take place.
- It is advisable to use NTP servers (e.g., from nl.pool.ntp.org) to avoid clock skew vulnerabilities.

### 9.6 Levels of Assurance

See section 10.3.

### 9.7 Local Session

- The DV is responsible for keeping track of the local End User session.
- This session MUST be terminated after at most **30 minutes** inactivity.
- The DV must recognize and ward off **replay attacks**.
- If the DV uses cookies to manage sessions, the **"Secure"** and **"HttpOnly"** parameters must be used.

### 9.8 RelayState

- DVs may provide a RelayState for their own session monitoring.
- The RD returns the RelayState without any verification.
- The monitoring of the content and integrity of the RelayState must be done by the service provider.
- The SAML standard uses a maximum of **80 characters** for the RelayState.

### 9.9 User Interaction

When a web service forwards an end user to an RD, AD, or BVD:
1. The end user must be redirected to the AD in the same screen where the user clicked "Log in to <AD>".
2. The end user must see a browser window with the full address bar (allowing URL/certificate inspection).
3. It is not allowed to invoke an RD, AD, or BVD website in a **frame or iframe**, or to embed it in any other way.

If the status in the Assertion is not successful or the user does not have the required LoA:
1. The DV or LC MUST immediately end the current session.
2. Should show an appropriate error message.

## 10 Type Definitions

### 10.1 Attribute Identifier Types

| Attribute | Identification-code | Remarks |
|-----------|-------------------|---------|
| BSN | urn:nl-eid-gdi:1.0:id:legacy-BSN | BSN. Encoded in 9-digits, padded with leading 0 if needed. Example: 123456789 or 012345678. |
| BSN | urn:nl-eid-gdi:1.0:id:BSN | Encrypted Identity |
| Pseudonym | urn:nl-eid-gdi:1.0:id:Pseudonym | Encrypted Pseudonym |

### 10.2 EntityID

Format: **`urn:nl-eid-gdi:1.0:<ROLE>:<OIN>:entities:<index>`**

| Attribute | Value | Remarks |
|-----------|-------|---------|
| `<OIN>` | The OIN of the organisation. | |
| `<ROLE>` | Indication of the role: AD, DV, BVD, LC, RD | |
| `<index>` | A number with 4 positions between 0000 and 8999 that can be selected by the participant or service provider to define different endpoints (in the metadata). Numbers between 9000 and 9999 are reserved for test systems. | |

### 10.3 Levels of Assurance

| DigiD 3.3 | DigiD | eTD | eIDAS | eID |
|-----------|-------|-----|-------|-----|
| - | - | 1 | - | - |
| urn:oasis:names:tc:SAML:2.0:ac:classes:PasswordProtectedTransport | Basis | 2 | Low | http://eID.logius.nl/LoA/basic |
| urn:oasis:names:tc:SAML:2.0:ac:classes:MobileTwoFactorContract | Midden | 2+ | Low | http://eidas.europa.eu/LoA/low |
| urn:oasis:names:tc:SAML:2.0:ac:classes:Smartcard | Substantieel | 3 | Substantial | http://eidas.europa.eu/LoA/substantial |
| urn:oasis:names:tc:SAML:2.0:ac:classes:SmartcardPKI | Hoog | 4 | High | http://eidas.europa.eu/LoA/high |
