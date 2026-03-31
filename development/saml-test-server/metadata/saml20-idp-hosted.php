<?php

$metadata['http://localhost:8080/simplesaml/saml2/idp/metadata.php'] = [
    'host' => '__DEFAULT__',
    'privatekey' => 'idp.pem',
    'certificate' => 'idp.crt',
    'auth' => 'example-static',
    'metadata.sign.enable' => true,
    'metadata.sign.privatekey' => 'idp.pem',
    'metadata.sign.certificate' => 'idp.crt',
    'SingleSignOnServiceBinding' => ["urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST"],
    // enabling sendartifact causes error when retrieving the metadata
    // 'saml20.sendartifact' => true,
];
