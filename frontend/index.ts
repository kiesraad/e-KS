import highlightActiveLinks from "./scripts/highlights-alerts/active-link";
import alertSuccess from "./scripts/highlights-alerts/alert-success";
import bsnInput from "./scripts/form-inputs/bsn-input";
import setupClickRow from "./scripts/table-interaction/click-row";
import countryCodeInput from "./scripts/form-inputs/country-input";
import dateInput from "./scripts/form-inputs/date-input";
import setupDirtyForms from "./scripts/form-inputs/dirty-form";
import setupFileImport from "./scripts/form-inputs/file-import";
import highlightRow from "./scripts/highlights-alerts/highlight-row";
import initialsInput from "./scripts/form-inputs/initials-input";
import localitySuggestions from "./scripts/form-inputs/locality-suggestions";
import addressLookup from "./scripts/form-inputs/lookup";
import setupModal from "./scripts/generic-ui/modal";
import setupOverlay from "./scripts/generic-ui/overlay";
import setupPositionPreview from "./scripts/form-inputs/position-preview";
import setupTextSearch from "./scripts/generic-ui/search";
import setupSelectAllCheckbox from "./scripts/form-inputs/select-all-checkbox";
import submitSamlLogin from "./scripts/generic-ui/saml-login";
import setupSortable from "./scripts/table-interaction/sortable";
import setupStickyNav from "./scripts/generic-ui/sticky-nav";
import setupRememberScroll from "./scripts/generic-ui/remember-scroll";

import "./styles/index.css";

// table interaction
setupClickRow();

// highlights and alerts
highlightActiveLinks();
alertSuccess();
highlightRow();

// form inputs
bsnInput();
countryCodeInput();
dateInput();
initialsInput();
setupFileImport();
addressLookup();
localitySuggestions();
setupPositionPreview();
setupSelectAllCheckbox();
setupDirtyForms();

// generic UI
setupStickyNav();
setupModal();
setupOverlay();
setupTextSearch();
setupSortable();
setupRememberScroll();
submitSamlLogin();
