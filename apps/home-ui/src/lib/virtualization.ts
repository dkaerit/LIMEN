export interface VirtualGridInput {
  itemCount: number;
  width: number;
  scrollOffset: number;
  viewportHeight: number;
  pinnedIndex?: number;
  overscanRows?: number;
}

export interface VirtualGridLayout {
  columns: number;
  gap: number;
  rowHeight: number;
  rowCount: number;
  totalHeight: number;
  visibleRows: number[];
}

export function columnsForWidth(width: number): number {
  if (width >= 1320) return 5;
  if (width >= 760) return 4;
  if (width >= 520) return 3;
  if (width >= 340) return 2;
  return 1;
}

export function calculateVirtualGrid({
  itemCount,
  width,
  scrollOffset,
  viewportHeight,
  pinnedIndex = -1,
  overscanRows = 2,
}: VirtualGridInput): VirtualGridLayout {
  const safeWidth = Math.max(width, 1);
  const columns = columnsForWidth(safeWidth);
  const gap = safeWidth >= 760 ? 14 : 10;
  const cardWidth = (safeWidth - gap * (columns - 1)) / columns;
  const rowHeight = cardWidth / 1.55 + gap;
  const rowCount = Math.ceil(Math.max(itemCount, 0) / columns);
  const safeScroll = Math.max(scrollOffset, 0);
  const firstVisibleRow = Math.floor(safeScroll / rowHeight);
  const lastVisibleRow = Math.ceil(
    (safeScroll + Math.max(viewportHeight, rowHeight)) / rowHeight,
  );
  const firstRow = Math.max(0, firstVisibleRow - overscanRows);
  const lastRow = Math.min(rowCount, lastVisibleRow + overscanRows);
  const visibleRows = Array.from(
    { length: Math.max(0, lastRow - firstRow) },
    (_, index) => firstRow + index,
  );

  if (pinnedIndex >= 0 && pinnedIndex < itemCount) {
    const pinnedRow = Math.floor(pinnedIndex / columns);
    if (!visibleRows.includes(pinnedRow)) {
      visibleRows.push(pinnedRow);
      visibleRows.sort((left, right) => left - right);
    }
  }

  return {
    columns,
    gap,
    rowHeight,
    rowCount,
    totalHeight: Math.max(0, rowCount * rowHeight - gap),
    visibleRows,
  };
}
