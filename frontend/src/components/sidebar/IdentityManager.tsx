// src/components/sidebar/IdentityManager.tsx
import { HelpCircle, DownloadCloud, Plus } from 'lucide-react';
import { Button } from '../common/Button';

export const IdentityManager = () => {
  return (
    <div className="bg-[#101010] border-[1.5px] border-[#1D1D1D] rounded-xl p-6 flex flex-col gap-5">
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold text-brand-text-light">
          Identity Manager
        </h3>
        <HelpCircle className="w-5 h-5 text-brand-text-dim cursor-pointer" />
      </div>

      {/* Кнопки Load / Create */}
      <div className="grid grid-cols-2 gap-3">
        <Button
          variant="secondary"
          Icon={DownloadCloud}
          fullWidth
        >
          Load
        </Button>
        <Button
          variant="primary"
          Icon={Plus}
          fullWidth
        >
          Create
        </Button>
      </div>
    </div>
  );
};