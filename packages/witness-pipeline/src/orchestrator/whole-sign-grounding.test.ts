import {describe,it,expect} from 'vitest';
import {deriveWholeSignHouses} from './engine-facts.js';
describe('deterministic whole-sign grounding',()=>{
 it('counts the ascendant as house one, giving Taurus opposite Scorpio',()=>{
  const houses=deriveWholeSignHouses({house_system:'whole-sign',lagna:{sign_index:7,longitude:222.1035}});
  expect(houses).toHaveLength(12);expect(houses[0]).toBe('1=Scorpio');
  expect(houses[6]).toBe('7=Taurus');expect(houses[8]).toBe('9=Cancer');
 });
 it('wraps across Pisces without changing the house numbering',()=>{
  const houses=deriveWholeSignHouses({house_system:'whole-sign',lagna:{sign_index:11,longitude:359.99}});
  expect(houses[1]).toBe('2=Aries');expect(houses[11]).toBe('12=Aquarius');
 });
 it.each([NaN,-1,360,210-0.0001])('rejects invalid or inconsistent longitude %s',longitude=>{
  expect(deriveWholeSignHouses({house_system:'whole-sign',lagna:{sign_index:7,longitude}})).toEqual([]);
 });
 it('does not assume whole-sign or silently resolve inconsistent inputs',()=>{
  expect(deriveWholeSignHouses({house_system:'Placidus',lagna:{sign_index:7,longitude:222}})).toEqual([]);
  expect(deriveWholeSignHouses({house_system:'whole-sign',lagna:{sign_index:7}})).toEqual([]);
 });
});
