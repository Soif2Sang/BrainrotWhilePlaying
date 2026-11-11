function (action) {
  if (location.protocol !== 'https:' || !(location.hostname === 'tiktok.com' || location.hostname.endsWith('.tiktok.com'))) return;
  const roots = [document];
  for (let i = 0; i < roots.length; i++) {
    for (const element of roots[i].querySelectorAll('*')) {
      if (element.shadowRoot) roots.push(element.shadowRoot);
    }
  }
  const query = selector => roots.flatMap(root => [...root.querySelectorAll(selector)]);
  const visible = el => {
    const r = el.getBoundingClientRect();
    return r.width > 0 && r.height > 0 && r.bottom > 0 && r.top < innerHeight;
  };
  const videos = query('video').filter(visible);
  const video = videos.sort((a, b) => {
    const distance = el => Math.abs(el.getBoundingClientRect().top + el.getBoundingClientRect().height / 2 - innerHeight / 2);
    return distance(a) - distance(b);
  })[0];
  const notify = text => {
    if (window.__padviewHUD) { window.__padviewHUD.notify(text); return text; }
    let toast = document.getElementById('padview-toast');
    if (!toast) {
      toast = document.createElement('div'); toast.id = 'padview-toast';
      Object.assign(toast.style, {position:'fixed',bottom:'24px',left:'16px',right:'16px',padding:'12px',background:'#161b26',color:'white',borderRadius:'10px',font:'14px system-ui',zIndex:'2147483647',pointerEvents:'none'});
      document.documentElement.appendChild(toast);
    }
    toast.textContent = text; toast.hidden = false;
    clearTimeout(toast.timer); toast.timer = setTimeout(() => { toast.hidden = true; }, 2200);
    return text;
  };
  if (action === 'next' || action === 'previous') {
    // shortcut: TikTok selectors can change; update these when site navigation changes.
    const direction = action === 'next' ? 'down' : 'up';
    const labels = action === 'next' ? ['Go to next video', 'Aller à la vidéo suivante', 'Next video'] : ['Go to previous video', 'Aller à la vidéo précédente', 'Previous video'];
    const button = query(`[data-e2e="arrow-${direction}"],${labels.map(label => `button[aria-label="${label}"]`).join(',')}`).find(el => visible(el) && !el.disabled);
    if (button) { button.click(); return; }
    let container = video?.parentElement;
    while (container && !(container.scrollHeight > container.clientHeight && /auto|scroll/.test(getComputedStyle(container).overflowY))) container = container.parentElement;
    const target = container || document.scrollingElement;
    if (target && target.scrollHeight > target.clientHeight) {
      target.scrollBy({ top: (action === 'next' ? 1 : -1) * Math.max(200, target.clientHeight * 0.85), behavior: 'smooth' });
    } else return notify('Aucune zone à faire défiler sur cette page.');
    return;
  }
  if (action === 'pause') {
    const button = query('button[aria-label="Pause"],button[aria-label="Play"],button[aria-label="Lecture"],button[aria-label="Lire"]').find(visible);
    if (button) { button.click(); return; }
  }
  if (!video) return notify('Aucune vidéo visible. Ouvre le fil Pour toi.');
  if (action === 'pause') {
    if (video.paused) video.play().catch(() => notify('Lecture bloquée : lance une première vidéo avec la souris.'));
    else video.pause();
  }
  if (action === 'mute') { video.muted = !video.muted; notify(video.muted ? 'Son coupé' : 'Son activé'); }
  if (action === 'quieter' || action === 'louder') {
    video.volume = Math.min(1, Math.max(0, video.volume + (action === 'louder' ? 0.1 : -0.1)));
    video.muted = false; notify(`Volume ${Math.round(video.volume * 100)} %`);
  }
}
