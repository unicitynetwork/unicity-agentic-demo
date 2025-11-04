import type { BalanceInfo } from '../../types';
import { AssetItem } from './AssetItem';

interface AssetListProps {
  balances: BalanceInfo[];
}

export const AssetList: React.FC<AssetListProps> = ({ balances }) => {
  return (
    <div className="bg-[#101010] border-[1.5px] border-[#1D1D1D] rounded-xl p-6">
      <h3 className="text-lg font-semibold mb-4 text-brand-text-light">
        My Assets
      </h3>
      
      <div className="flex flex-col">
        {balances.map((balance) => (
          <AssetItem key={balance.asset_id} asset={balance} />
        ))}
      </div>
    </div>
  );
};