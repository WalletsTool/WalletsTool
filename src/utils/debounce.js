/**
 * 防抖函数
 * @param {Function} func 要防抖的函数
 * @param {number} delay 延迟时间（毫秒）
 * @param {boolean} immediate 是否立即执行
 * @returns {Function} 防抖后的函数
 */
export function debounce(func, delay = 300, immediate = false) {
  let timeoutId;
  
  return function executedFunction(...args) {
    const callNow = immediate && !timeoutId;
    
    clearTimeout(timeoutId);
    
    timeoutId = setTimeout(() => {
      timeoutId = null;
      if (!immediate) func.apply(this, args);
    }, delay);
    
    if (callNow) func.apply(this, args);
  };
}

/**
 * 节流函数
 * @param {Function} func 要节流的函数
 * @param {number} limit 时间间隔（毫秒）
 * @returns {Function} 节流后的函数
 */
export function throttle(func, limit = 300) {
  let inThrottle;
  
  return function executedFunction(...args) {
    if (!inThrottle) {
      func.apply(this, args);
      inThrottle = true;
      setTimeout(() => inThrottle = false, limit);
    }
  };
}

/**
 * 创建防抖的响应式更新函数
 * @param {Function} updateFn 更新函数
 * @param {number} delay 防抖延迟
 * @returns {Function} 防抖后的更新函数
 */
export function createDebouncedUpdate(updateFn, delay = 300) {
  return debounce(updateFn, delay);
}

/**
 * 创建节流的响应式更新函数
 * @param {Function} updateFn 更新函数
 * @param {number} limit 节流间隔
 * @returns {Function} 节流后的更新函数
 */
export function createThrottledUpdate(updateFn, limit = 300) {
  return throttle(updateFn, limit);
}