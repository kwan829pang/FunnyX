const cards = [
  ["Gateway", "8080", "#67e8f9"],
  ["Client Center", "8083", "#a78bfa"],
  ["Session Tokens", "8082", "#f6c453"],
  ["Admin API", "18300", "#fda085"],
  ["Webhook", "8084", "#fb7185"],
  ["Message Center", "8085", "#34d399"],
  ["Config", "8090", "#94a3b8"],
  ["Core Engine", "18200", "#60a5fa"],
  ["Payment gate", "18100", "#fbbf24"],
  ["Company A", "18102", "#22d3ee"],
  ["Market bot", "18103", "#c084fc"],
  ["Client Web", "UI", "#e879f9"],
];

const flows = {
  shop: {
    line: [
      ["Player", "Pick a coin pack"],
      ["Payment gate", "Fiat checkout"],
    ],
    ask: "Paid?",
    yes: [
      ["Webhook", "Settles once"],
      ["Wallet", "Credited once"],
    ],
    no: [["Order", "No credit"]],
  },
  trade: {
    line: [
      ["Book", "Already open"],
      ["Player", "Place a price"],
    ],
  },
  deal: {},
};

const flowCanvas = document.getElementById("flow-canvas");
let currentFlow = "shop";

function roundRect(ctx, x, y, w, h, r) {
  ctx.beginPath();
  ctx.moveTo(x + r, y);
  ctx.arcTo(x + w, y, x + w, y + h, r);
  ctx.arcTo(x + w, y + h, x, y + h, r);
  ctx.arcTo(x, y + h, x, y, r);
  ctx.arcTo(x, y, x + w, y, r);
  ctx.closePath();
}

function wrapLines(ctx, text, maxWidth) {
  const words = text.split(" ");
  const lines = [];
  let line = "";
  words.forEach((word) => {
    const test = line ? `${line} ${word}` : word;
    if (ctx.measureText(test).width > maxWidth && line) {
      lines.push(line);
      line = word;
    } else {
      line = test;
    }
  });
  if (line) lines.push(line);
  return lines;
}

function drawCentered(ctx, text, cx, cy, maxWidth, font) {
  ctx.font = font;
  const lines = wrapLines(ctx, text, maxWidth);
  const lineHeight = 16;
  const top = cy - ((lines.length - 1) * lineHeight) / 2;
  lines.forEach((line, i) => ctx.fillText(line, cx, top + i * lineHeight));
}

function arrowHead(ctx, x, y, color, angle) {
  ctx.save();
  ctx.translate(x, y);
  ctx.rotate(angle);
  ctx.fillStyle = color;
  ctx.beginPath();
  ctx.moveTo(0, 0);
  ctx.lineTo(-8, -4.5);
  ctx.lineTo(-8, 4.5);
  ctx.closePath();
  ctx.fill();
  ctx.restore();
}

function strokeArrow(ctx, points, color) {
  ctx.strokeStyle = color;
  ctx.lineWidth = 2;
  ctx.beginPath();
  ctx.moveTo(points[0][0], points[0][1]);
  for (let i = 1; i < points.length; i++) ctx.lineTo(points[i][0], points[i][1]);
  ctx.stroke();
  const prev = points[points.length - 2];
  const end = points[points.length - 1];
  arrowHead(ctx, end[0], end[1], color, Math.atan2(end[1] - prev[1], end[0] - prev[0]));
}

function drawBox(ctx, x, y, w, h, who, what, color) {
  ctx.save();
  roundRect(ctx, x, y, w, h, 8);
  ctx.fillStyle = "#121933";
  ctx.fill();
  ctx.lineWidth = 2;
  ctx.strokeStyle = color;
  ctx.stroke();
  ctx.clip();
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.fillStyle = "#9aa6c7";
  drawCentered(ctx, who, x + w / 2, y + h * 0.34, w - 16, "12px Segoe UI, system-ui, sans-serif");
  ctx.fillStyle = "#eef2ff";
  const titleSize = w < 120 ? 13 : 15;
  drawCentered(ctx, what, x + w / 2, y + h * 0.66, w - 16, `600 ${titleSize}px Segoe UI, system-ui, sans-serif`);
  ctx.restore();
}

function drawDiamond(ctx, cx, cy, size, label) {
  ctx.beginPath();
  ctx.moveTo(cx, cy - size / 2);
  ctx.lineTo(cx + size / 2, cy);
  ctx.lineTo(cx, cy + size / 2);
  ctx.lineTo(cx - size / 2, cy);
  ctx.closePath();
  ctx.fillStyle = "#121933";
  ctx.fill();
  ctx.lineWidth = 2;
  ctx.strokeStyle = "#f6c453";
  ctx.stroke();
  ctx.fillStyle = "#eef2ff";
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  let fontSize = 14;
  ctx.font = `700 ${fontSize}px Segoe UI, system-ui, sans-serif`;
  while (fontSize > 10 && ctx.measureText(label).width > size * 0.72) {
    fontSize -= 1;
    ctx.font = `700 ${fontSize}px Segoe UI, system-ui, sans-serif`;
  }
  ctx.fillText(label, cx, cy);
}

function fitFlow(width, nLeft, nRight) {
  const boxes = nLeft + nRight;
  const gaps = nLeft + nRight;
  let arrow = 22;
  let diamond = 96;
  let boxW = 168;
  const inner = Math.max(120, width - 16);
  const usedNow = () => boxes * boxW + diamond + arrow * gaps;
  if (usedNow() > inner) boxW = (inner - diamond - arrow * gaps) / boxes;
  if (boxW < 110) {
    arrow = 14;
    diamond = 74;
    boxW = 168;
    if (usedNow() > inner) boxW = (inner - diamond - arrow * gaps) / boxes;
  }
  let used = usedNow();
  if (used > inner) {
    const scale = inner / used;
    boxW *= scale;
    diamond *= scale;
    arrow *= scale;
    used = inner;
  }
  return { arrow, diamond, boxW, start: (width - used) / 2 };
}

function paintFlow() {
  const wrap = flowCanvas.parentElement;
  const width = wrap.clientWidth;
  const height = wrap.clientHeight;
  if (width < 20 || height < 20) return;
  const dpr = Math.min(window.devicePixelRatio || 1, 2);
  flowCanvas.width = Math.floor(width * dpr);
  flowCanvas.height = Math.floor(height * dpr);
  flowCanvas.style.width = `${width}px`;
  flowCanvas.style.height = `${height}px`;
  const ctx = flowCanvas.getContext("2d");
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.clearRect(0, 0, width, height);

  const flow = flows[currentFlow];
  if (currentFlow === "trade") {
    paintTrade(ctx, width, height);
    return;
  }
  if (currentFlow === "deal") {
    paintDeal(ctx, width, height);
    return;
  }
  const nRight = Math.max(flow.yes.length, flow.no.length);
  const { arrow, diamond, boxW, start } = fitFlow(width, flow.line.length, nRight);
  const boxH = Math.min(86, Math.max(62, height * 0.28));
  const cy = height / 2;
  const yesY = cy - boxH * 0.78;
  const noY = cy + boxH * 0.78;

  let x = start;
  const left = flow.line.map((item) => {
    const box = { x, y: cy - boxH / 2, w: boxW, h: boxH, who: item[0], what: item[1] };
    x += boxW + arrow;
    return box;
  });
  const diamondCx = x + diamond / 2;
  const rightX = diamondCx + diamond / 2 + arrow;
  const placeRow = (items, rowY) =>
    items.map((item, i) => ({
      x: rightX + i * (boxW + arrow),
      y: rowY - boxH / 2,
      w: boxW,
      h: boxH,
      who: item[0],
      what: item[1],
    }));
  const yes = placeRow(flow.yes, yesY);
  const no = placeRow(flow.no, noY);

  left.forEach((box, i) => {
    const nextX = i < left.length - 1 ? left[i + 1].x : diamondCx - diamond / 2;
    strokeArrow(ctx, [[box.x + box.w, cy], [nextX - 1, cy]], "#67e8f9");
  });
  const fork = (boxes, color, label, rowY, above) => {
    const stub = diamondCx + diamond / 2 + arrow * 0.42;
    strokeArrow(
      ctx,
      [
        [diamondCx + diamond / 2, cy],
        [stub, cy],
        [stub, rowY],
        [boxes[0].x - 1, rowY],
      ],
      color,
    );
    boxes.forEach((box, i) => {
      if (i < boxes.length - 1) {
        strokeArrow(ctx, [[box.x + box.w, rowY], [boxes[i + 1].x - 1, rowY]], color);
      }
    });
    ctx.fillStyle = color;
    ctx.font = "700 12px Segoe UI, system-ui, sans-serif";
    ctx.textAlign = "left";
    ctx.textBaseline = above ? "bottom" : "top";
    ctx.fillText(label, stub + 6, above ? cy - 6 : cy + 6);
  };
  fork(yes, "#34d399", "Yes", yesY, true);
  fork(no, "#fb7185", "No", noY, false);

  left.forEach((box) => drawBox(ctx, box.x, box.y, box.w, box.h, box.who, box.what, "#67e8f9"));
  drawDiamond(ctx, diamondCx, cy, diamond, flow.ask);
  yes.forEach((box) => drawBox(ctx, box.x, box.y, box.w, box.h, box.who, box.what, "#34d399"));
  no.forEach((box) => drawBox(ctx, box.x, box.y, box.w, box.h, box.who, box.what, "#fb7185"));
  flowCanvas.setAttribute("aria-label", `${flow.ask} flowchart`);
}

function paintTrade(ctx, width, height) {
  const gapX = Math.max(14, Math.min(28, width * 0.02));
  const gapY = Math.max(16, Math.min(28, height * 0.06));
  const boxW = Math.min(200, (width - 20 - gapX * 3) / 4);
  const boxH = Math.min(78, (height - 20 - gapY * 2) / 3);
  const usedW = boxW * 4 + gapX * 3;
  const usedH = boxH * 3 + gapY * 2;
  const x0 = (width - usedW) / 2;
  const y0 = (height - usedH) / 2;
  const col = (i) => x0 + i * (boxW + gapX);
  const row = (i) => y0 + i * (boxH + gapY);
  const box = (c, r, who, what, color) => ({
    x: col(c),
    y: row(r),
    w: boxW,
    h: boxH,
    who,
    what,
    color,
  });
  const book = box(0, 1, "Book", "Already open", "#67e8f9");
  const place = box(1, 1, "Player", "Place a price", "#67e8f9");
  const active = box(2, 1, "Active", "Pending on the book", "#f6c453");
  const partial = box(3, 0, "Partial", "Rest stays pending", "#67e8f9");
  const filled = box(3, 1, "Filled", "Fully matched", "#34d399");
  const cancelled = box(3, 2, "Cancelled", "Player stops it", "#fb7185");
  const rejected = box(1, 2, "Rejected", "Engine refuses it", "#fb7185");
  const mid = (item) => item.y + item.h / 2;

  strokeArrow(ctx, [[book.x + book.w, mid(book)], [place.x - 1, mid(place)]], "#67e8f9");
  strokeArrow(ctx, [[place.x + place.w, mid(place)], [active.x - 1, mid(active)]], "#f6c453");
  strokeArrow(ctx, [[place.x + place.w / 2, place.y + place.h], [rejected.x + rejected.w / 2, rejected.y - 1]], "#fb7185");
  strokeArrow(ctx, [[active.x + active.w, mid(active)], [filled.x - 1, mid(filled)]], "#34d399");
  const forkX = active.x + active.w + gapX * 0.45;
  strokeArrow(
    ctx,
    [
      [active.x + active.w, mid(active)],
      [forkX, mid(active)],
      [forkX, mid(partial)],
      [partial.x - 1, mid(partial)],
    ],
    "#67e8f9",
  );
  strokeArrow(
    ctx,
    [
      [active.x + active.w, mid(active)],
      [forkX, mid(active)],
      [forkX, mid(cancelled)],
      [cancelled.x - 1, mid(cancelled)],
    ],
    "#fb7185",
  );

  [book, place, active, partial, filled, cancelled, rejected].forEach((item) => {
    drawBox(ctx, item.x, item.y, item.w, item.h, item.who, item.what, item.color);
  });
  flowCanvas.setAttribute("aria-label", "Trade order statuses");
}

function paintDeal(ctx, width, height) {
  const captionH = 24;
  const laneGap = 8;
  const innerH = height - captionH - 4;
  const placeH = innerH * 0.3;
  const takeH = innerH * 0.44;
  const cancelH = innerH - placeH - takeH;
  const left = 8;
  const bandW = width - 16;
  const gapX = Math.max(12, Math.min(22, width * 0.016));
  const mid = (item) => item.y + item.h / 2;
  const link = (boxes, color) => {
    boxes.forEach((box, i) => {
      if (i < boxes.length - 1) {
        strokeArrow(ctx, [[box.x + box.w, mid(box)], [boxes[i + 1].x - 1, mid(boxes[i + 1])]], color);
      }
    });
  };

  laneBand(ctx, left, 2, bandW, placeH - laneGap, "User A places");
  laneBand(ctx, left, 2 + placeH, bandW, takeH - laneGap, "User B takes");
  laneBand(ctx, left, 2 + placeH + takeH, bandW, cancelH - laneGap, "User A cancels while open");

  const placeBoxes = (() => {
    const h = Math.min(58, placeH - 46);
    const count = 4;
    const w = Math.min(180, (bandW - 28 - gapX * (count - 1)) / count);
    const x = left + (bandW - (count * w + (count - 1) * gapX)) / 2;
    const y = 2 + 28 + Math.max(0, (placeH - laneGap - 28 - h) / 2);
    return placeRow(x, y, w, h, gapX, [
      ["User A", "Pick game and items", "#67e8f9"],
      ["User A", "Enter the amount", "#67e8f9"],
      ["Open", "Placed, 1 PLT fee", "#f6c453"],
      ["Notice created", "Poster + webhook", "#34d399"],
    ]);
  })();
  link(placeBoxes, "#f6c453");
  placeBoxes.forEach((box) => drawBox(ctx, box.x, box.y, box.w, box.h, box.who, box.what, box.color));

  const takeTop = 2 + placeH;
  const stackGap = 8;
  const takeBoxH = Math.min(54, (takeH - 44 - stackGap) / 2);
  const takeCount = 4;
  const takeW = Math.min(176, (bandW - 28 - gapX * (takeCount - 1)) / takeCount);
  const takeUsed = takeCount * takeW + (takeCount - 1) * gapX;
  const takeX = left + (bandW - takeUsed) / 2;
  const stackH = takeBoxH * 2 + stackGap;
  const takeY = takeTop + 28 + Math.max(0, (takeH - laneGap - 28 - stackH) / 2);
  const takeSteps = placeRow(takeX, takeY + (stackH - takeBoxH) / 2, takeW, takeBoxH, gapX, [
    ["User B", "Take the open deal", "#67e8f9"],
    ["Platform", "Move want coins", "#f6c453"],
    ["Corp Server", "Transfer the offer", "#22d3ee"],
  ]);
  const outcomes = [
    ["Completed", "Notice to both + webhook", "#34d399"],
    ["Failed", "Wallets back + notice", "#fb7185"],
  ].map((item, i) => ({
    x: takeX + 3 * (takeW + gapX),
    y: takeY + i * (takeBoxH + stackGap),
    w: takeW,
    h: takeBoxH,
    who: item[0],
    what: item[1],
    color: item[2],
  }));
  link(takeSteps, "#f6c453");
  const settle = takeSteps[2];
  const forkX = settle.x + settle.w + gapX * 0.45;
  outcomes.forEach((box) => {
    const y = mid(box);
    const points = Math.abs(y - mid(settle)) < 2
      ? [[settle.x + settle.w, mid(settle)], [box.x - 1, y]]
      : [[settle.x + settle.w, mid(settle)], [forkX, mid(settle)], [forkX, y], [box.x - 1, y]];
    strokeArrow(ctx, points, box.color);
  });
  [...takeSteps, ...outcomes].forEach((box) => drawBox(ctx, box.x, box.y, box.w, box.h, box.who, box.what, box.color));

  const cancelTop = 2 + placeH + takeH;
  const cancelHBox = Math.min(56, cancelH - 44);
  const cancelCount = 3;
  const cancelW = Math.min(200, (bandW - 28 - gapX * (cancelCount - 1)) / cancelCount);
  const cancelUsed = cancelCount * cancelW + (cancelCount - 1) * gapX;
  const cancelX = left + (bandW - cancelUsed) / 2;
  const cancelY = cancelTop + 28 + Math.max(0, (cancelH - laneGap - 28 - cancelHBox) / 2);
  const cancelBoxes = placeRow(cancelX, cancelY, cancelW, cancelHBox, gapX, [
    ["User A", "Cancel the open deal", "#fb7185"],
    ["Cancelled", "1 PLT fee stays", "#fb7185"],
    ["Notice cancelled", "Poster + webhook", "#fb7185"],
  ]);
  link(cancelBoxes, "#fb7185");
  cancelBoxes.forEach((box) => drawBox(ctx, box.x, box.y, box.w, box.h, box.who, box.what, box.color));

  ctx.fillStyle = "#9aa6c7";
  ctx.font = "13px Segoe UI, system-ui, sans-serif";
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.fillText("Message Center writes the user notice and posts the partner webhook.", width / 2, height - 12);
  flowCanvas.setAttribute("aria-label", "Deal place, take, and notices");
}

function showFlow(name) {
  currentFlow = name;
  paintFlow();
}
new ResizeObserver(paintFlow).observe(flowCanvas.parentElement);
document.querySelectorAll("#path-tabs button").forEach((btn) => {
  btn.addEventListener("click", () => {
    document.querySelectorAll("#path-tabs button").forEach((b) => b.classList.remove("on"));
    btn.classList.add("on");
    showFlow(btn.dataset.flow);
  });
});
showFlow("shop");

const mapCanvas = document.getElementById("map-canvas");

function laneBand(ctx, x, y, w, h, title) {
  roundRect(ctx, x, y, w, h, 14);
  ctx.fillStyle = "rgba(255,255,255,0.03)";
  ctx.fill();
  ctx.lineWidth = 1;
  ctx.strokeStyle = "rgba(255,255,255,0.06)";
  ctx.stroke();
  ctx.fillStyle = "#f6c453";
  ctx.font = "700 12px Segoe UI, system-ui, sans-serif";
  ctx.textAlign = "left";
  ctx.textBaseline = "top";
  ctx.fillText(title, x + 14, y + 10);
}

function placeRow(x, y, boxW, boxH, gap, items) {
  return items.map((item, i) => ({
    x: x + i * (boxW + gap),
    y,
    w: boxW,
    h: boxH,
    who: item[0],
    what: item[1],
    color: item[2],
  }));
}

function paintMap() {
  const wrap = mapCanvas.parentElement;
  const width = wrap.clientWidth;
  const height = wrap.clientHeight;
  if (width < 20 || height < 20) return;
  const dpr = Math.min(window.devicePixelRatio || 1, 2);
  mapCanvas.width = Math.floor(width * dpr);
  mapCanvas.height = Math.floor(height * dpr);
  mapCanvas.style.width = `${width}px`;
  mapCanvas.style.height = `${height}px`;
  const ctx = mapCanvas.getContext("2d");
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.clearRect(0, 0, width, height);

  const captionH = 28;
  const laneGap = 8;
  const innerH = height - captionH - 6;
  const userH = innerH * 0.48;
  const adminH = innerH * 0.24;
  const hookH = innerH - userH - adminH;
  const left = 8;
  const bandW = width - 16;

  laneBand(ctx, left, 4, bandW, userH - laneGap, "User");
  laneBand(ctx, left, 4 + userH, bandW, adminH - laneGap, "Admin");
  laneBand(ctx, left, 4 + userH + adminH, bandW, hookH - laneGap, "Corp Server callback");

  const userTop = 4;
  const labelH = 28;
  const stackGap = 8;
  const gapX = Math.max(14, Math.min(26, width * 0.018));
  const boxW = Math.min(168, (bandW - 28 - gapX * 3) / 4);
  const boxH = Math.min(56, (userH - labelH - 18 - stackGap * 2) / 3);
  const stackH = boxH * 3 + stackGap * 2;
  const rowY = userTop + labelH + Math.max(0, (userH - laneGap - labelH - stackH) / 2);
  const rowX = left + (bandW - (boxW * 4 + gapX * 3)) / 2;
  const mid = (item) => item.y + item.h / 2;
  const steps = placeRow(rowX, rowY + (stackH - boxH) / 2, boxW, boxH, gapX, [
    ["User", "Client Web", "#67e8f9"],
    ["Gateway", "Via Session Tokens", "#a78bfa"],
    ["Client Center", "Routes the call", "#67e8f9"],
  ]);
  const stackX = rowX + 3 * (boxW + gapX);
  const branches = [
    ["Payment gate", "Shop fiat", "#f6c453"],
    ["Core Engine", "Trade book", "#60a5fa"],
    ["Corp Server", "Deal and wallet", "#22d3ee"],
  ].map((item, i) => ({
    x: stackX,
    y: rowY + i * (boxH + stackGap),
    w: boxW,
    h: boxH,
    who: item[0],
    what: item[1],
    color: item[2],
  }));
  steps.forEach((box, i) => {
    if (i < steps.length - 1) {
      strokeArrow(ctx, [[box.x + box.w, mid(box)], [steps[i + 1].x - 1, mid(steps[i + 1])]], "#67e8f9");
    }
  });
  const center = steps[2];
  const forkX = center.x + center.w + gapX * 0.45;
  branches.forEach((box) => {
    const y = mid(box);
    const points = Math.abs(y - mid(center)) < 2
      ? [[center.x + center.w, mid(center)], [box.x - 1, y]]
      : [[center.x + center.w, mid(center)], [forkX, mid(center)], [forkX, y], [box.x - 1, y]];
    strokeArrow(ctx, points, box.color);
  });
  [...steps, ...branches].forEach((box) => drawBox(ctx, box.x, box.y, box.w, box.h, box.who, box.what, box.color));

  const drawLine = (laneY, laneHeight, items, color) => {
    const h = Math.min(58, laneHeight - 40);
    const w = Math.min(168, (bandW - 28 - gapX * (items.length - 1)) / items.length);
    const used = items.length * w + (items.length - 1) * gapX;
    const x = left + (bandW - used) / 2;
    const y = laneY + labelH + Math.max(0, (laneHeight - laneGap - labelH - h) / 2);
    const boxes = placeRow(x, y, w, h, gapX, items);
    boxes.forEach((box, i) => {
      if (i < boxes.length - 1) {
        strokeArrow(ctx, [[box.x + box.w, mid(box)], [boxes[i + 1].x - 1, mid(boxes[i + 1])]], color);
      }
    });
    boxes.forEach((box) => drawBox(ctx, box.x, box.y, box.w, box.h, box.who, box.what, box.color));
  };

  drawLine(4 + userH, adminH, [
    ["Admin", "Admin Panel", "#fda085"],
    ["Admin API", "Skips Gateway", "#fda085"],
    ["Core Engine", "Opens the book", "#60a5fa"],
  ], "#fda085");
  drawLine(4 + userH + adminH, hookH, [
    ["Corp Server", "Sends the callback", "#22d3ee"],
    ["Webhook", "Not through Gateway", "#fb7185"],
    ["Client Center", "Settles once", "#67e8f9"],
    ["Message Center", "Notices both sides", "#34d399"],
  ], "#fb7185");

  ctx.fillStyle = "#9aa6c7";
  ctx.font = "13px Segoe UI, system-ui, sans-serif";
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.fillText("Config answers a private ping. It is not on these calls.", width / 2, height - 12);
  mapCanvas.setAttribute("aria-label", "How requests move");
}

new ResizeObserver(paintMap).observe(mapCanvas.parentElement);
paintMap();

const charts = {
  shop: {
    note: "Buying Platform Token or a company coin. Fiat goes through the payment gate. This is not a deposit.",
    rail: [
      ["wait", "Pending", "Player", "Creates a shop order", "Order stays pending. Wallet unchanged."],
      ["wait", "Payment gate", "Payment gate", "Takes fiat at checkout", "Partner payment opens. Order still pending."],
      ["go", "Paid", "Webhook → Client Center", "Settles the same event once", "Order paid. Wallet credited once."],
    ],
    fork: [
      ["stop", "Failed", "Payment gate", "Reports pay failed", "Order failed. Wallet unchanged."],
      ["stop", "Cancelled", "Player or partner", "Cancels the checkout", "Order cancelled. Wallet unchanged."],
      ["stop", "Expired", "Platform", "24h with no pay", "Order expired. Wallet unchanged."],
    ],
  },
  wallet: {
    note: "Moving a game coin with the company game server. Not the payment gate. Not used for shop, deals, or the book.",
    rail: [
      ["wait", "Pending", "Player", "Asks to deposit or withdraw", "Money txn pending on that game account."],
      ["wait", "Partner", "Company server", "Accepts the amount", "Partner txn id stored. Wallet still waiting."],
      ["go", "Completed", "Webhook → Client Center", "Applies the callback", "Deposit credits the wallet. Withdraw debits it."],
    ],
    fork: [
      ["stop", "Failed", "Company server", "Cannot finish", "Txn failed. Wallet unchanged."],
      ["stop", "Rejected", "Company server", "Refuses the amount", "Txn rejected. Wallet unchanged."],
      ["stop", "Cancelled", "Player or partner", "Stops the request", "Txn cancelled. Wallet unchanged."],
    ],
  },
  market: {
    note: "A company opens a trading pair. Players do not deposit to create it.",
    rail: [
      ["wait", "Submitted", "Corp user", "Submits a market pair", "Market request stored. Book not open."],
      ["wait", "Pending lock", "Platform", "Locks company coin for the pool", "Game balance locked. Pool still pending."],
      ["go", "Active", "Admin", "Approves and opens the engine", "Lock moves into the pool. Core Engine lists the pair."],
    ],
    fork: [
      ["stop", "Rejected", "Admin", "Declines the pair", "Lock released back to the company balance."],
    ],
  },
  deal: {
    note: "User A picks a game and items, enters the amount, and places an open deal. User B takes it. Settle changes the status. Each step writes a notice and a partner webhook.",
    rail: [
      ["wait", "Pick", "User A", "Picks a game and its items", "From the game list and the item list."],
      ["wait", "Amount", "User A", "Enters the required amount", "The want side is a platform wallet coin."],
      ["wait", "Open", "User A", "Places the deal", "Status stays open. Notice created goes to the poster, plus a partner webhook."],
      ["wait", "Settle", "Platform + Corp Server", "Moves want coins, then Transfer", "This step changes the status."],
      ["go", "Completed", "Corp Server", "Transfer succeeds", "Notice completed goes to both users, plus a partner webhook."],
    ],
    fork: [
      ["stop", "Failed", "Corp Server", "Transfer fails", "Wallets reverse. Notice failed goes to both users, plus a partner webhook."],
      ["stop", "Cancelled", "User A", "Cancels while the deal is open", "The 1 PLT fee stays. Notice cancelled goes to the poster, plus a partner webhook."],
    ],
  },
  trade: {
    note: "A price order stays pending on the book. Balances are already on the platform.",
    rail: [
      ["wait", "Active", "Player", "Places a price", "Order stays pending until it matches or the player cancels."],
      ["wait", "Partial", "Core Engine", "Fills part of the quantity", "The rest stays pending on the book."],
      ["go", "Filled", "Core Engine", "Fills what is left", "Nothing remains on the book."],
    ],
    fork: [
      ["stop", "Cancelled", "Player", "Cancels while it is still pending", "The open quantity leaves the book."],
      ["stop", "Rejected", "Core Engine", "Refuses the order", "It never rests on the book."],
    ],
  },
  auth: {
    note: "How a call is allowed before any of the actions above.",
    rail: [
      ["wait", "Public", "Player", "Registers, logs in, or starts OAuth", "No session yet."],
      ["go", "Token", "Session Token Server", "Issues a session token", "Client Web holds the token."],
      ["go", "Session", "Gateway", "Checks the token, then forwards", "Client Center or the engine receives the call."],
    ],
    fork: [
      ["stop", "Invalid", "Gateway", "Token missing or rejected", "Call stops. Nothing else changes."],
    ],
  },
};

const statusCanvas = document.getElementById("status-canvas");
const chartNote = document.getElementById("chart-note");
let currentChart = "shop";
const statusColor = { wait: "#f6c453", go: "#34d399", stop: "#fb7185" };

function drawStatusBox(ctx, x, y, w, h, status, who, action, effect, color) {
  ctx.save();
  roundRect(ctx, x, y, w, h, 10);
  ctx.fillStyle = "#121933";
  ctx.fill();
  ctx.lineWidth = 2;
  ctx.strokeStyle = color;
  ctx.stroke();
  ctx.clip();
  const pad = 12;
  ctx.textAlign = "left";
  ctx.textBaseline = "top";
  ctx.fillStyle = "#eef2ff";
  ctx.font = "700 15px Segoe UI, system-ui, sans-serif";
  ctx.fillText(status, x + pad, y + 10);
  ctx.fillStyle = color;
  ctx.font = "12px Segoe UI, system-ui, sans-serif";
  ctx.fillText(who, x + pad, y + 32);
  ctx.fillStyle = "#eef2ff";
  ctx.font = "14px Segoe UI, system-ui, sans-serif";
  const actionLines = wrapLines(ctx, action, w - pad * 2).slice(0, 2);
  actionLines.forEach((line, i) => ctx.fillText(line, x + pad, y + 52 + i * 16));
  ctx.fillStyle = "#9aa6c7";
  ctx.font = "12px Segoe UI, system-ui, sans-serif";
  const effectY = y + 54 + actionLines.length * 16;
  wrapLines(ctx, effect, w - pad * 2).slice(0, 2).forEach((line, i) => ctx.fillText(line, x + pad, effectY + i * 15));
  ctx.restore();
}

function paintStatus() {
  const wrap = statusCanvas.parentElement;
  const width = wrap.clientWidth;
  const height = wrap.clientHeight;
  if (width < 20 || height < 20) return;
  const dpr = Math.min(window.devicePixelRatio || 1, 2);
  statusCanvas.width = Math.floor(width * dpr);
  statusCanvas.height = Math.floor(height * dpr);
  statusCanvas.style.width = `${width}px`;
  statusCanvas.style.height = `${height}px`;
  const ctx = statusCanvas.getContext("2d");
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.clearRect(0, 0, width, height);

  const chart = charts[currentChart];
  const rail = chart.rail;
  const fork = chart.fork;
  const gapX = Math.max(22, Math.min(36, width * 0.028));
  const cols = Math.max(rail.length, fork.length, 1);
  const boxW = Math.min(230, (width - 16 - gapX * (cols - 1)) / cols);
  const gapY = 18;
  const boxH = Math.min(128, (height - gapY - 8) / 2);
  const midY = (item) => item.y + item.h / 2;
  const row = (items, y) => {
    const used = items.length * boxW + Math.max(0, items.length - 1) * gapX;
    const x0 = (width - used) / 2;
    return items.map((item, i) => ({
      x: x0 + i * (boxW + gapX),
      y,
      w: boxW,
      h: boxH,
      kind: item[0],
      status: item[1],
      who: item[2],
      action: item[3],
      effect: item[4],
    }));
  };
  const top = row(rail, Math.max(4, (height / 2 - gapY / 2 - boxH) / 2));
  const bottom = row(fork, height / 2 + gapY / 2);
  top.forEach((box, i) => {
    if (i < top.length - 1) {
      strokeArrow(ctx, [[box.x + box.w, midY(box)], [top[i + 1].x - 1, midY(top[i + 1])]], statusColor[box.kind]);
    }
  });
  [...top, ...bottom].forEach((box) => {
    drawStatusBox(ctx, box.x, box.y, box.w, box.h, box.status, box.who, box.action, box.effect, statusColor[box.kind]);
  });
  statusCanvas.setAttribute("aria-label", chart.note);
}

function drawChart(name) {
  currentChart = name;
  chartNote.textContent = charts[name].note;
  paintStatus();
}
new ResizeObserver(paintStatus).observe(statusCanvas.parentElement);

document.querySelectorAll("#status-tabs button").forEach((btn) => {
  btn.addEventListener("click", () => {
    document.querySelectorAll("#status-tabs button").forEach((b) => b.classList.remove("on"));
    btn.classList.add("on");
    drawChart(btn.dataset.chart);
  });
});
drawChart("shop");

document.getElementById("grid").innerHTML = cards
  .map(
    ([name, portLabel, color]) =>
      `<article class="card"><span class="swatch" style="background:${color}"></span><b>${name}</b><i>${portLabel}</i></article>`,
  )
  .join("");
