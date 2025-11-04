// src/components/common/Button.tsx
import React from 'react';
import type { LucideIcon } from 'lucide-react';

// Определяем варианты стилей
type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'icon';

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  children: React.ReactNode;
  variant?: ButtonVariant;
  Icon?: LucideIcon;
  fullWidth?: boolean;
}

export const Button: React.FC<ButtonProps> = ({
  children,
  variant = 'secondary',
  Icon,
  fullWidth = false,
  className = '',
  ...props
}) => {
  const baseStyle = `
    flex items-center justify-center gap-2
    text-sm font-semibold
    transition-all duration-200
    disabled:opacity-50
  `;

  const variantStyles = {
    primary: `
      px-4 py-2.5 rounded-lg
      bg-brand-green text-brand-text-dark
      hover:bg-brand-green-hover
    `,
    secondary: `
      px-4 py-2.5 rounded-lg
      bg-brand-bg-dark text-brand-text-light
      border border-brand-bg-border
      hover:bg-brand-bg-border
    `,
    ghost: `
      px-4 py-2.5 rounded-lg
      bg-transparent text-brand-text-dim
      hover:bg-brand-bg-light hover:text-brand-text-light
    `,
    icon: `
      flex-none w-7 h-7 rounded-full p-0
    `,
  };

  const widthStyle = fullWidth ? 'w-full' : '';

  return (
    <button
      className={`${baseStyle} ${variantStyles[variant]} ${widthStyle} ${className}`}
      {...props}
    >
      {Icon && <Icon className="w-4 h-4" />}
      <span>{children}</span>
    </button>
  );
};