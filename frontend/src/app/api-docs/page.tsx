import type { Metadata } from 'next'

export const metadata: Metadata = {
  title: 'API Documentation | StelloVault',
  description: 'Interactive OpenAPI documentation for the StelloVault REST API.',
}

export default function ApiDocsPage() {
  return (
    <main className="min-h-screen">
      {/* Swagger UI via CDN — no build-time dependency */}
      <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css" />
      <div id="swagger-ui" />
      <script
        dangerouslySetInnerHTML={{
          __html: `
            (function() {
              var script = document.createElement('script');
              script.src = 'https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js';
              script.onload = function() {
                SwaggerUIBundle({
                  url: '/openapi.yaml',
                  dom_id: '#swagger-ui',
                  presets: [SwaggerUIBundle.presets.apis, SwaggerUIBundle.SwaggerUIStandalonePreset],
                  layout: 'BaseLayout',
                  deepLinking: true,
                });
              };
              document.body.appendChild(script);
            })();
          `,
        }}
      />
    </main>
  )
}
