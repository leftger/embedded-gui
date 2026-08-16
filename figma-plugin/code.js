// Embedded GUI Sync - Official Figma to KDL Exporter
// Translates Figma Frames, AutoLayouts, Components, and Vector Paths into embedded-gui declarative KDL.

figma.showUI(__html__, { width: 440, height: 580 });

function toSnakeCase(str) {
  return str
    .replace(/([a-z])([A-Z])/g, '$1_$2')
    .replace(/[\s\-]+/g, '_')
    .toLowerCase();
}

function sanitizeId(name) {
  const clean = name.replace(/[^a-zA-Z0-9_]/g, '_');
  return toSnakeCase(clean) || 'widget';
}

function escapeKdlString(str) {
  return str.replace(/\\/g, '\\\\').replace(/"/g, '\\"');
}

function inferWidgetType(node) {
  const name = node.name.toLowerCase();
  if (name.includes('button') || name.includes('btn')) return 'button';
  if (name.includes('slider')) return 'slider';
  if (name.includes('toggle') || name.includes('switch')) return 'toggle';
  if (name.includes('check') || name.includes('checkbox')) return 'checkbox';
  if (name.includes('gauge') || name.includes('tach') || name.includes('scale')) return 'scale';
  if (name.includes('progress') || name.includes('battery')) return 'progress';
  if (name.includes('spinner') || name.includes('loader') || name.includes('busy')) return 'busy_wheel';
  if (name.includes('scope') || name.includes('plotter') || name.includes('chart') || name.includes('wave')) return 'plotter';
  if (name.includes('status') || name.includes('clock') || name.includes('header_bar')) return 'status_bar';
  if (name.includes('roller') || name.includes('picker')) return 'roller';
  if (name.includes('dropdown') || name.includes('select')) return 'dropdown';
  if (name.includes('panel') || name.includes('card')) return 'panel';
  if (node.type === 'TEXT') return 'label';
  if (node.type === 'VECTOR') return 'vector_path';
  return 'panel';
}

function extractText(node) {
  if (node.type === 'TEXT') {
    return node.characters;
  }
  if ('children' in node) {
    for (const child of node.children) {
      if (child.type === 'TEXT') {
        return child.characters;
      }
    }
  }
  return node.name;
}

function convertFigmaToKdl(node) {
  if (!node) {
    return '// Please select a Frame or Component in Figma to export.';
  }

  const screenId = (node.name.replace(/[^a-zA-Z0-9]/g, '') || 'FigmaScreen');
  const width = Math.round(node.width) || 320;
  const height = Math.round(node.height) || 240;

  let kdl = `screen id="${screenId}" width=${width} height=${height} theme="dark" {\n`;

  const isAutoLayout = 'layoutMode' in node && (node.layoutMode === 'HORIZONTAL' || node.layoutMode === 'VERTICAL');
  const gap = isAutoLayout ? Math.round(node.itemSpacing || 4) : 4;
  const padding = isAutoLayout ? Math.round(node.paddingLeft || 6) : 6;

  const children = ('children' in node) ? node.children : [node];
  const numChildren = children.length;

  let colsTrack = '1fr 1fr';
  let rowsTrack = '1fr 1fr';

  if (isAutoLayout && node.layoutMode === 'VERTICAL') {
    colsTrack = '1fr';
    const rowList = [];
    for (const c of children) {
      const h = Math.round(c.height);
      rowList.push(h > 0 ? `${h}px` : '1fr');
    }
    rowsTrack = rowList.join(' ') || '1fr';
  } else if (isAutoLayout && node.layoutMode === 'HORIZONTAL') {
    rowsTrack = '1fr';
    const colList = [];
    for (const c of children) {
      const w = Math.round(c.width);
      colList.push(w > 0 ? `${w}px` : '1fr');
    }
    colsTrack = colList.join(' ') || '1fr';
  }

  kdl += `    grid cols="${colsTrack}" rows="${rowsTrack}" gap=${gap} padding=${padding} {\n`;

  children.forEach((child, index) => {
    let col = 0;
    let row = index;

    if (isAutoLayout) {
      if (node.layoutMode === 'HORIZONTAL') {
        col = index;
        row = 0;
      } else {
        col = 0;
        row = index;
      }
    } else {
      // 2-column fallback grid
      col = index % 2;
      row = Math.floor(index / 2);
    }

    const widgetType = inferWidgetType(child);
    const id = sanitizeId(child.name);
    const text = escapeKdlString(extractText(child));

    switch (widgetType) {
      case 'label':
        kdl += `        label id="${id}" text="${text}" col=${col} row=${row}\n`;
        break;
      case 'button':
        kdl += `        button id="${id}" text="${text}" style="accent" col=${col} row=${row}\n`;
        break;
      case 'toggle':
        kdl += `        toggle id="${id}" label="${text}" checked=true col=${col} row=${row}\n`;
        break;
      case 'checkbox':
        kdl += `        checkbox id="${id}" label="${text}" checked=false col=${col} row=${row}\n`;
        break;
      case 'slider':
        kdl += `        slider id="${id}" min=0 max=100 value=50 col=${col} row=${row}\n`;
        break;
      case 'progress':
        kdl += `        progress id="${id}" value=0.75 col=${col} row=${row}\n`;
        break;
      case 'scale':
        kdl += `        scale id="${id}" mode="radial" min=0.0 max=100.0 value=65.0 major_ticks=5 col=${col} row=${row}\n`;
        break;
      case 'busy_wheel':
        kdl += `        busy_wheel id="${id}" active=true col=${col} row=${row}\n`;
        break;
      case 'plotter':
        kdl += `        plotter id="${id}" mode="sine" col=${col} row=${row}\n`;
        break;
      case 'status_bar':
        kdl += `        status_bar id="${id}" time="12:00" col=${col} row=${row}\n`;
        break;
      case 'roller':
        kdl += `        roller id="${id}" selected=1 col=${col} row=${row} {\n`;
        kdl += `            option "Item 1"\n            option "Item 2"\n            option "Item 3"\n`;
        kdl += `        }\n`;
        break;
      case 'dropdown':
        kdl += `        dropdown id="${id}" selected=0 col=${col} row=${row} {\n`;
        kdl += `            option "Option A"\n            option "Option B"\n`;
        kdl += `        }\n`;
        break;
      case 'panel':
      default:
        kdl += `        panel id="${id}" style="card" col=${col} row=${row}\n`;
        break;
    }
  });

  kdl += `    }\n`;
  kdl += `}\n`;

  return kdl;
}

function updateSelection() {
  const selection = figma.currentPage.selection;
  if (selection.length === 0) {
    figma.ui.postMessage({
      type: 'NO_SELECTION',
      message: 'Select a Frame, Component, or Group to generate KDL markup.',
    });
    return;
  }

  const rootNode = selection[0];
  const kdl = convertFigmaToKdl(rootNode);

  figma.ui.postMessage({
    type: 'KDL_GENERATED',
    kdl: kdl,
    nodeName: rootNode.name,
    width: Math.round(rootNode.width),
    height: Math.round(rootNode.height),
    childrenCount: ('children' in rootNode) ? rootNode.children.length : 1,
  });
}

figma.on('selectionchange', () => {
  updateSelection();
});

figma.ui.onmessage = (msg) => {
  if (msg.type === 'NOTIFY') {
    figma.notify(msg.message);
  }
};

// Initial run
updateSelection();
