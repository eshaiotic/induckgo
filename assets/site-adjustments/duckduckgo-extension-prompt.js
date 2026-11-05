// InDuckGo: DuckDuckGo's own pages offer a browser extension to every Chromium browser. InDuckGo has the
// protections built in and runs no extensions, so on duckduckgo.com the offer is hidden instead of the page
// being told something untrue about this browser.
(function () {
  var host = location.hostname;
  if (host !== 'duckduckgo.com' && !host.endsWith('.duckduckgo.com')) return;
  var sheet = new CSSStyleSheet();
  sheet.replaceSync(
    '[data-ssg-id*="extension"], [data-testid*="extension"], [data-testid*="Extension"],' +
    ' li:has(> [data-ssg-id*="extension"]), li:has(> [data-testid*="extension"]),' +
    ' a[href*="chromewebstore.google.com"], a[href*="chrome.google.com/webstore"],' +
    ' a[href*="addons.mozilla.org"], a[href*="microsoftedge.microsoft.com/addons"]' +
    ' { display: none !important; }'
  );
  document.adoptedStyleSheets = document.adoptedStyleSheets.concat([sheet]);
})();
