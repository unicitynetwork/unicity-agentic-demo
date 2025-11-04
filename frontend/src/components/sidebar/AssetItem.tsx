// src/components/sidebar/AssetItem.tsx
import type { BalanceInfo } from '../../types';

interface AssetItemProps {
  asset: BalanceInfo;
}

export const AssetItem: React.FC<AssetItemProps> = ({ asset }) => {
  return (
    <div className="
      flex items-center justify-between 
      py-3
      first:pt-0
      last:pb-0
    ">
      <div className="flex items-center gap-3">
        <img
          src="/icons/unicityLogo.svg"
          alt={`${asset.asset_id} icon`}
          className="w-8 h-8 rounded-full bg-brand-bg-dark"
        />
        <div>
          <p className="text-sm font-medium text-brand-text-light">{asset.asset_id}</p>
          <p className="text-xs text-brand-text-dim">{asset.asset_id}</p>
        </div>
      </div>

      <p className="text-sm font-medium text-brand-text-light">
        {asset.balance}
      </p>
    </div>
  );
};