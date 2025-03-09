import { clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs) {
	return twMerge(clsx(inputs));
}
export function formatDate(date, format) {
	const pad = (n) => n.toString().padStart(2, '0');
	
	const year = date.getFullYear();
	const month = pad(date.getMonth() + 1);
	const day = pad(date.getDate());
	const hours = pad(date.getHours());
	const minutes = pad(date.getMinutes());
	const seconds = pad(date.getSeconds());
  
	return format
	  .replace('yyyy', year)
	  .replace('MM', month)
	  .replace('dd', day)
	  .replace('HH', hours)
	  .replace('mm', minutes)
	  .replace('ss', seconds);
  }
  
  export const flyAndScale = (
	node,
	params = { y: -8, x: 0, start: 0.95, duration: 150 }
  ) => {
	const style = getComputedStyle(node);
	const transform = style.transform === 'none' ? '' : style.transform;
  
	const scaleConversion = (valueA, scaleA, scaleB) => {
	  const [minA, maxA] = scaleA;
	  const [minB, maxB] = scaleB;
  
	  const percentage = (valueA - minA) / (maxA - minA);
	  const valueB = percentage * (maxB - minB) + minB;
  
	  return valueB;
	};
  
	const styleToString = (style) => {
	  return Object.keys(style).reduce((str, key) => {
		if (style[key] === undefined) return str;
		return str + `${key}:${style[key]};`;
	  }, '');
	};
  
	return {
	  duration: params.duration ?? 200,
	  delay: 0,
	  css: (t) => {
		const y = scaleConversion(t, [0, 1], [params.y ?? 5, 0]);
		const x = scaleConversion(t, [0, 1], [params.x ?? 0, 0]);
		const scale = scaleConversion(t, [0, 1], [params.start ?? 0.95, 1]);
  
		return styleToString({
		  transform: `${transform} translate3d(${x}px, ${y}px, 0) scale(${scale})`,
		  opacity: t
		});
	  },
	  easing: cubicOut
	};
  };