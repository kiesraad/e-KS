export default function submitSamlLogin() {
  const form = document.getElementById("saml-login-form");

  if (!(form instanceof HTMLFormElement)) {
    return;
  }

  form.requestSubmit();
}
