(() => {
  if (location.protocol !== 'https:' || !(location.hostname === 'tiktok.com' || location.hostname.endsWith('.tiktok.com'))) return;
  const mount = () => {
    const style = document.createElement('style');
    // shortcut: these selectors follow TikTok's feed layout; the full-site toggle restores the normal page.
    style.textContent = `
      html.padview-compact, html.padview-compact body { background:#090a0c!important; overflow:hidden!important; }
      html.padview-compact * { scrollbar-width:none!important; }
      html.padview-compact [class*="SideNav"], html.padview-compact [data-e2e="top-header"] { visibility:hidden!important; pointer-events:none!important; }
      html.padview-compact main[id^="main-content-homepage"] { position:fixed!important; inset:0!important; width:100%!important; max-width:none!important; margin:0!important; padding:0!important; z-index:100!important; background:#090a0c!important; }
      html.padview-compact main[id^="main-content-homepage"] article { position:relative!important; box-sizing:border-box!important; width:100%!important; max-width:none!important; height:100vh!important; padding:0!important; margin:0!important; }
      html.padview-compact main[id^="main-content-homepage"] > :not(article):not(:has(article)) { opacity:0!important; pointer-events:none!important; }
      html.padview-compact [id^="media-card-"] { position:absolute!important; inset:0!important; width:100%!important; height:100%!important; max-width:none!important; margin:0!important; padding:0!important; }
      html.padview-compact video { object-fit:contain!important; }
    `;
    document.head.appendChild(style);document.documentElement.classList.add('padview-compact');
  };
  if(document.readyState==='loading') document.addEventListener('DOMContentLoaded',mount,{once:true});else mount();
})();
