<script setup lang="ts">
/**
 * 隐形窗口缩放感应区
 * 放置在最外层容器中，用于捕捉窗口边缘的拖拽操作
 */
defineProps<{
  onResize: (direction: string, e: MouseEvent) => void
  onReset: () => void
}>()
</script>
<template>
  <div class="resize-edge n" @mousedown="onResize('n', $event)"></div>
  <div class="resize-edge s" @mousedown="onResize('s', $event)"></div>
  <div class="resize-edge e" @mousedown="onResize('e', $event)"></div>
  <div class="resize-edge w" @mousedown="onResize('w', $event)"></div>
  <div class="resize-corner nw" @mousedown="onResize('nw', $event)"></div>
  <div class="resize-corner ne" @mousedown="onResize('ne', $event)"></div>
  <div class="resize-corner sw" @mousedown="onResize('sw', $event)"></div>
  <div class="resize-corner se" @mousedown="onResize('se', $event)"></div>
</template>

<style scoped>
/* 缩放辅助区样式：全透明但可以捕捉鼠标事件 */
.resize-edge,
.resize-corner {
  position: absolute;
  z-index: 1000;
}

/* 顶部感应区 */

.resize-edge.n {
  top: 0;
  left: 10px;
  right: 10px;
  height: 5px;
  cursor: n-resize;
}

/* 底部感应区 */

.resize-edge.s {
  bottom: 0;
  left: 10px;
  right: 10px;
  height: 5px;
  cursor: s-resize;
}

/* 右侧感应区 */
.resize-edge.e {
  right: 0;
  top: 10px;
  bottom: 10px;
  width: 5px;
  cursor: e-resize;
}

/* 左侧感应区 */
.resize-edge.w {
  left: 0;
  top: 10px;
  bottom: 10px;
  width: 5px;
  cursor: w-resize;
}

/* 四个角落感应区 */

.resize-corner.nw {
  top: 0;
  left: 0;
  width: 10px;
  height: 10px;
  cursor: nw-resize;
}

.resize-corner.ne {
  top: 0;
  right: 0;
  width: 10px;
  height: 10px;
  cursor: ne-resize;
}

.resize-corner.sw {
  bottom: 0;
  left: 0;
  width: 10px;
  height: 10px;
  cursor: sw-resize;
}

.resize-corner.se {
  bottom: 0;
  right: 0;
  width: 10px;
  height: 10px;
  cursor: se-resize;
}
</style>
