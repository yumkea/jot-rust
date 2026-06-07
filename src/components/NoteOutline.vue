<script setup lang="ts">
import { computed } from 'vue'

interface Heading {
  level: number
  text: string
  pos: number
}

const props = defineProps<{
  headings: Heading[]
  isOpen: boolean
  width?: number // 可选的宽度覆盖
}>()

const emit = defineEmits<{
  (e: 'select', pos: number): void
}>()

const numberedHeadings = computed(() => {
  const levels = [0, 0, 0, 0] // 对应 H2, H3, H4, H5 (最多4级)
  return props.headings.map((heading) => {
    // 一级标题 (H1) 不计编号
    if (heading.level <= 1) {
      return { ...heading, number: '' }
    }
    // 仅计算 H2-H5 (4个层级)
    const currentLevel = heading.level
    if (currentLevel > 5) {
      return { ...heading, number: '' }
    }

    const idx = currentLevel - 2 // H2 -> 0, H3 -> 1, ...

    // 增加当前层级计数
    levels[idx]++

    // 重置所有子层级
    for (let i = idx + 1; i < levels.length; i++) {
      levels[i] = 0
    }

    // 生成编号字符串 (例如 1. 或 1.1.)
    const numberStr = levels.slice(0, idx + 1).join('.') + '.'

    return {
      ...heading,
      number: numberStr
    }
  })
})
</script>

<template>
  <aside
    class="note-outline scrollbar-hide"
    :class="{ 'is-open': isOpen }"
    :style="{
      width: isOpen ? (width ? width + 'px' : '160px') : '0'
    }"
  >
    <div class="outline-container" :style="{ width: width ? width + 'px' : '160px' }">
      <div
        v-for="(heading, index) in numberedHeadings"
        :key="index"
        class="outline-item"
        :class="[`level-${heading.level}`]"
        @click="emit('select', heading.pos)"
      >
        <span class="outline-text">
          <span v-if="heading.number" class="numbering">{{ heading.number }}</span>
          {{ heading.text }}
        </span>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.note-outline {
  width: 0;
  padding: 20px 0;
  overflow-y: auto;
  /* 允许垂直方向滚动 */
  overflow-x: hidden;
  /* 隐藏水平滚动 */
  flex-shrink: 0;
}

.note-outline.is-open {
  width: 160px;
}

/* 隐藏滚动条但保留滚动功能 */
.note-outline::-webkit-scrollbar {
  display: none;
}

.note-outline {
  -ms-overflow-style: none;
  /* IE and Edge */
  scrollbar-width: none;
  /* Firefox */
}

.outline-container {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 2px 1px 1px 1px;
  position: relative;
  /* 宽度由内联样式动态设置，不再使用固定值 */
  box-sizing: border-box;
}

/* 垂直左侧线 */
.outline-container::before {
  content: '';
  position: absolute;
  left: 4px;
  top: 5px;
  bottom: 5px;
  width: 1px;
  background: var(--border-color);
  z-index: 0;
}

.outline-item {
  position: relative;
  cursor: pointer;
  padding: 2px 0;
  transition: all 0.2s ease;
  z-index: 1;
}

.outline-text {
  font-size: var(--panel-font-size-item);
  color: var(--text-low);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  display: block;
}

.numbering {
  font-family: inherit;
  margin-right: 4px;
  opacity: 0.8;
  font-variant-numeric: tabular-nums;
  font-size: var(--panel-font-size-small);
}

.outline-item:hover .outline-text {
  color: var(--text-main);
}

.level-1 {
  padding-left: 12px;
}

.level-1 .outline-text {
  font-weight: 500;
  font-size: var(--panel-font-size-label);
}

.level-2 {
  padding-left: 20px;
}

.level-3 {
  padding-left: 28px;
}

.level-4 {
  padding-left: 36px;
}

.level-5 {
  padding-left: 44px;
}

.level-6 {
  padding-left: 52px;
}

/* 激活标记线：只有在 level-1 时显示在主轴线上 */
.outline-item::after {
  content: '';
  position: absolute;
  left: 7px;
  top: 50%;
  transform: translateY(-50%);
  width: 3px;
  /* 指示线加粗 */
  height: 0;
  background: var(--accent-color);
  /* 颜色与光标同步 */
  transition: height 0.2s ease;
}

.outline-item:hover::after {
  height: 80%;
}

.scrollbar-hide::-webkit-scrollbar {
  display: none;
}

.scrollbar-hide {
  -ms-overflow-style: none;
  scrollbar-width: none;
}
</style>
