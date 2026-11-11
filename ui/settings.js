const { invoke } = window.__TAURI__.core;
const labels = { help:'Afficher / masquer le panneau', mode:'Interaction / mode jeu', visibility:'Afficher / masquer la fenêtre', previous:'Remonter', next:'Descendre', pause:'Lecture / pause', quieter:'Baisser le volume', louder:'Monter le volume', mute:'Couper / rétablir le son', corner:'Changer de coin', size:'Changer de taille', compact:'Vue compacte / site complet', quit:'Quitter BrainrotWhilePlaying' };
const $ = selector => document.querySelector(selector);
let current, draft, defaults, captureAction = null, captureTimer, busy = false;
function pretty(shortcut) {
  const names={NumpadAdd:'Pavé +',NumpadSubtract:'Pavé −',NumpadMultiply:'Pavé *',NumpadDivide:'Pavé /',NumpadDecimal:'Pavé .',NumpadEnter:'Pavé Entrée',Space:'Espace',ArrowUp:'↑',ArrowDown:'↓',ArrowLeft:'←',ArrowRight:'→'};
  return shortcut.split('+').map(key=>names[key] || key.replace(/^Numpad/,'Pavé ').replace(/^Key/,'').replace(/^Digit/,'')).join(' + ');
}
function message(text, error=false) { $('#message').textContent=text; $('#message').classList.toggle('error',error); }
function changed() { return JSON.stringify(draft)!==JSON.stringify(current?.bindings); }
function render() {
  if (!draft) return;
  $('#bindings').replaceChildren(...draft.map(binding=>{
    const row=document.createElement('div'); row.className='row';
    const label=document.createElement('span'); label.className='label'; label.textContent=labels[binding.action];
    const button=document.createElement('button'); button.className='key'; button.textContent=captureAction===binding.action?'À toi…':pretty(binding.shortcut);
    button.setAttribute('aria-label',`${labels[binding.action]} : ${pretty(binding.shortcut)}, modifier`);
    button.classList.toggle('changed',current.bindings.find(b=>b.action===binding.action)?.shortcut!==binding.shortcut);
    button.classList.toggle('listening',captureAction===binding.action);
    button.disabled=busy || !!captureAction;
    button.addEventListener('click',()=>beginCapture(binding.action)); row.append(label,button); return row;
  }));
  $('#capture').hidden=!captureAction;
  $('#save').disabled=busy || !!captureAction || !changed();
  for(const selector of ['#close','#mode','#reset']) $(selector).disabled=busy || !!captureAction;
  const help=draft.find(b=>b.action==='help');
  $('#hint').textContent=`${pretty(help.shortcut)} : panneau · Verr Num activé · Modificateurs acceptés`;
}
window.updateSettings = state => {
  const backendChanged=current && JSON.stringify(current.bindings)!==JSON.stringify(state.bindings);
  current=state;
  document.documentElement.classList.toggle('toolbar-only',!state.help);
  const badge=$('#listening');
  badge.textContent=state.capturing?'⌨ Saisie…':state.listening?'⌨ Pavé actif':'⌨ Indisponible';
  badge.classList.toggle('active',state.listening);
  badge.title='Les raccourcis restent actifs pendant le jeu.';
  $('#toolbar-listening').textContent=badge.textContent;
  $('#drag-handle').disabled=state.game_mode || state.capturing;
  $('#mode').lastElementChild.textContent='Reprendre le jeu ↗';
  if(!draft || backendChanged) draft=structuredClone(state.bindings);
  if(state.error) message(state.error,true);
  render();
};
async function beginCapture(action) {
  busy=true; render(); message('');
  try {
    await invoke('capture_key',{active:true}); captureAction=action;
    captureTimer=setTimeout(()=>finishCapture(null),30000);
  } catch(error) {message(String(error),true);}
  finally {busy=false;render();if(captureAction) $('#cancel').focus();}
}
async function finishCapture(shortcut) {
  if(!captureAction || busy) return;
  const action=captureAction; busy=true; clearTimeout(captureTimer);
  try {
    await invoke('capture_key',{active:false});
    if(shortcut) {
      const duplicate=draft.find(b=>b.action!==action && b.shortcut.toLowerCase()===shortcut.toLowerCase());
      if(duplicate) message(`Déjà utilisé pour « ${labels[duplicate.action]} ».`,true);
      else {draft.find(b=>b.action===action).shortcut=shortcut;message('Modification prête à enregistrer.');}
    }
    captureAction=null;
  } catch(error) {message(String(error),true);}
  finally {busy=false;render();}
}
let pendingKey = null;
document.addEventListener('keydown',event=>{
  if(!captureAction || busy) return;
  event.preventDefault();event.stopPropagation();
  if(event.code==='Escape') {pendingKey=null;finishCapture(null);return;}
  if(/^(Control|Alt|Shift|Meta)(Left|Right)$/.test(event.code) || event.repeat) return;
  const modifiers=[];
  if(event.ctrlKey) modifiers.push('Control'); if(event.altKey) modifiers.push('Alt');
  if(event.shiftKey) modifiers.push('Shift'); if(event.metaKey) modifiers.push('Super');
  pendingKey={code:event.code,shortcut:[...modifiers,event.code].join('+')};
},true);
document.addEventListener('keyup',event=>{
  if(captureAction && pendingKey?.code===event.code) {
    event.preventDefault();const shortcut=pendingKey.shortcut;pendingKey=null;finishCapture(shortcut);
  }
},true);
window.addEventListener('blur',()=>{pendingKey=null;if(captureAction) finishCapture(null);});
$('#cancel').addEventListener('click',()=>finishCapture(null));
async function action(name) {
  busy=true;render();
  try {await invoke('overlay_action',{action:name});}
  catch(error) {message(String(error),true);}
  finally {busy=false;render();}
}
$('#close').addEventListener('click',()=>action('help'));
$('#mode').addEventListener('click',()=>action('mode'));
for(const selector of ['#drag-handle','#panel-drag']) $(selector).addEventListener('pointerdown',event=>{
  if(event.button!==0 || captureAction || current?.game_mode) return;
  event.preventDefault();
  invoke('drag_window').catch(error=>message(String(error),true));
});
$('#reset').addEventListener('click',()=>{draft=structuredClone(defaults);message('Touches par défaut prêtes à enregistrer.');render();});
$('#save').addEventListener('click',async()=>{
  busy=true;render();
  try {await invoke('save_bindings',{bindings:draft});message('Raccourcis enregistrés.');}
  catch(error) {message(String(error),true);}
  finally {busy=false;render();}
});
(async()=>{
  try {
    await invoke('capture_key',{active:false});
    const state=await invoke('settings_state'); defaults=state.defaults;window.updateSettings(state);
  } catch(error) {message(String(error),true);}
})();
