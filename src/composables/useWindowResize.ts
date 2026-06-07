import { invoke } from '@tauri-apps/api/core'

/**
 * 窗口缩放逻辑 Composables
 */
export function useWindowResize(): {
  startResize: (direction: string, e: MouseEvent) => void
} {
  /**
   * 全方位窗口缩放逻辑
   * @param direction 缩放方向：'n'(北), 's'(南), 'e'(东), 'w'(西) 及其组合
   */
  const startResize = (direction: string, e: MouseEvent): void => {
    e.preventDefault()
    e.stopPropagation()

    const startX = e.screenX
    const startY = e.screenY
    const startWidth = window.outerWidth
    const startHeight = window.outerHeight
    const startWinX = window.screenX
    const startWinY = window.screenY

    const onMouseMove = (moveEvent: MouseEvent): void => {
      const deltaX = moveEvent.screenX - startX
      const deltaY = moveEvent.screenY - startY

      let newWidth = startWidth
      let newHeight = startHeight
      let newX = startWinX
      let newY = startWinY

      if (direction.includes('e')) newWidth = Math.max(320, startWidth + deltaX)
      if (direction.includes('s')) newHeight = Math.max(250, startHeight + deltaY)

      if (direction.includes('w')) {
        const targetWidth = Math.max(320, startWidth - deltaX)
        newWidth = targetWidth
        if (targetWidth > 320 || deltaX < 0) {
          newX = startWinX + (startWidth - targetWidth)
        }
      }

      if (direction.includes('n')) {
        const targetHeight = Math.max(250, startHeight - deltaY)
        newHeight = targetHeight
        if (targetHeight > 250 || deltaY < 0) {
          newY = startWinY + (startHeight - targetHeight)
        }
      }

      invoke('resize_window', { width: newWidth, height: newHeight, x: newX, y: newY })
    }

    const onMouseUp = (): void => {
      window.removeEventListener('mousemove', onMouseMove)
      window.removeEventListener('mouseup', onMouseUp)
    }

    window.addEventListener('mousemove', onMouseMove)
    window.addEventListener('mouseup', onMouseUp)
  }

  return {
    startResize
  }
}
