{{/* Generate full hostname */}}
{{- define "fullHostname" -}}
  {{- if .Values.subdomain -}}
    {{- printf "%s.%s" .Values.subdomain .Values.hostname -}}
  {{- else -}}
    {{- .Values.hostname -}}
  {{- end -}}
{{- end -}}
