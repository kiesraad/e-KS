<?php

$metadata['urn:nl-eid-gdi:1.0:DV:00000004185618890000:entities:9000'] = [
    'host' => '__DEFAULT__',
    'name' => [
        'en' => 'Electronic Candidate Nomination System',
        'nl' => 'Elektronisch Kandidaatstellingssysteem',
    ],
    'OrganizationName' => [
        'en' => 'Electoral Council of the Netherlands',
        'nl' => 'Kiesraad',
    ],
    'certificate' => 'e-ks.crt',
    'AssertionConsumerService' => [
        [
            'Location' => 'http://localhost:3000/saml/acs',
            'Binding' => 'urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST',
        ],
    ],
    'SingleLogoutService' => [
        [
            'Location' => 'http://localhost:3000/saml/logout',
            'Binding' => 'urn:oasis:names:tc:SAML:2.0:bindings:HTTP-Redirect',
        ],
    ],
];
