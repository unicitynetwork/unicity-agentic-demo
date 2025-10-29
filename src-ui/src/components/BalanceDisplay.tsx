import React from 'react';
import { BalanceInfo } from '../types';

interface BalanceDisplayProps {
  balances: BalanceInfo[];
}

const BalanceDisplay: React.FC<BalanceDisplayProps> = ({ balances }) => {
  const usdtBalance = balances.find(b => b.asset_id === 'USDT');
  const otherBalances = balances.filter(b => b.asset_id !== 'USDT');

  return (
    <div className="balance-card">
      <div className="balance-title">Total Balance</div>
      {usdtBalance && (
        <div className="balance-amount">
          {parseFloat(usdtBalance.balance).toLocaleString()} USDT
        </div>
      )}
      
      {otherBalances.length > 0 && (
        <div className="asset-list">
          {otherBalances.map((balance) => (
            <div key={balance.asset_id} className="asset-item">
              <span className="asset-name">{balance.asset_id}</span>
              <span className="asset-balance">{balance.balance}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

export default BalanceDisplay;