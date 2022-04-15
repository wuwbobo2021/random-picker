// by wuwbobo2021 <https://github.com/wuwbobo2021>, <wuwbobo@outlook.com>
// If you have found bugs in this program, please pull an issue, or contact me.

#include "picker.h"
#include <iostream>
using namespace RandomPicker;

void Picker::pick(unsigned int amount, std::vector<unsigned int>& result)
{
	if (amount == 0 || m_table.count() == 0) return;
	if (!m_table.repetitive_picking && amount > m_table.count())
		throw std::invalid_argument("Picker::pick(): invalid amount.");
	
	if (! m_flag_testing) this->draw();
	double width = m_grid[m_table.count()]; if (width == 0) return;
	
	double r; unsigned int n;
	bool picked[m_table.count()] = {false};
	result.clear();
	for (unsigned int i = 0; i < amount; i++) {
		r = this->random_value(width);
		n = this->determine(r);
		if (! m_table.repetitive_picking) {
			if (picked[n]) {i--; continue;}
			picked[n] = true;
		}
		result.push_back(n);
	}
}

void Picker::pick(unsigned int amount, std::vector<std::string>& result)
{
	if (amount == 0 || m_table.count() == 0) return;
	
	std::vector<unsigned int> vect;
	this->pick(amount, vect);
	for (unsigned int i = 0; i < amount; i++)
		result.push_back(m_table[vect[i]].name());
}

void Picker::test(unsigned int times, unsigned int amount, Table& result)
{
	if (amount == 0 || times == 0) return;
	
	unsigned int stat[m_table.count()] = {0};
	std::vector<unsigned int> vect;
	this->draw(); m_flag_testing = true;
	for (unsigned int i = 0; i < times; i++) {
		this->pick(amount, vect);
		for (unsigned int i = 0; i < amount; i++)
			stat[vect[i]]++;
	}
	
	m_flag_testing = false;
	result.clear();
	unsigned int cnt = m_table.count();
	for (unsigned int i = 0; i < cnt; i++) {
		Item item(m_table[i].name(), stat[i]);
		result.item(item);
	}
}

void Picker::draw()
{
	m_grid.clear();
	if (m_table.is_empty()) return;
	
	Table tbl = m_table;
	if (tbl.power_inversed) tbl.inverse();
	
	unsigned int cnt = m_table.count();
	double cur = 0;
	for (unsigned int i = 0; i < cnt; i++) {
		m_grid.push_back(cur);
		cur += tbl[i].value();
	}
	m_grid.push_back(cur);
}

unsigned int Picker::determine(double val) const
{
	unsigned int sz = m_grid.size();
	if (sz <= 2) return 0;
	
	const double* p = m_grid.data();
	if (val == p[sz - 1]) return sz - 2;
	for (unsigned int i = 0; i < sz - 1; i++)
		if (val >= p[i] && val < p[i + 1]) return i;
	
	return 0; //it should be impossible
}

void Picker::calculate(unsigned int pick_amount, Table& result) const
{
	if (pick_amount == 0 || m_table.count() == 0) return;
	if (!m_table.repetitive_picking && pick_amount > m_table.count())
		throw std::invalid_argument("Picker::calculate(): invalid amount.");
		
	Table tbl = m_table;
	if (tbl.power_inversed) tbl.inverse();
	
	double width = 0;
	for (unsigned int i = 0; i < tbl.count(); i++)
		width += tbl[i].value();
	
	if (tbl.repetitive_picking || pick_amount == 1) {
		tbl.scale(1.0 / width);
		result = tbl;
	} else if (pick_amount == tbl.count()) {
		for (unsigned int i = 0; i < tbl.count(); i++)
			tbl[i].value(1);
		result = tbl;
	} else {
		double pro[tbl.count()] = {0};
		unsigned int stack[pick_amount] = {0}; unsigned int dep = 0;
		bool picked[tbl.count()] = {false};
		bool flag_back = false;
		
		while (true) {
			unsigned int i = stack[dep]; bool pre_picked = picked[i];
			if (!pre_picked && !flag_back) {
				picked[i] = true;
				
				double cur_pro = 1.0, tmp_width = width;
				for (unsigned int j = 0; j <= dep; j++) { //do such calculations every time to avoid lose of accuracy
					cur_pro *= tbl[stack[j]].value() / tmp_width;
					tmp_width -= tbl[stack[j]].value();
				}
				pro[i] += cur_pro;
			}
			if (!pre_picked && !flag_back && dep < pick_amount - 1) { //go down
				dep++; stack[dep] = 0;
			} else if (i < tbl.count() - 1) { //go right
				if (!pre_picked || flag_back) picked[i] = false;
				stack[dep]++; flag_back = false;
			} else { //go back or break
				bool first_loop = true;
				while (stack[dep] >= tbl.count() - 1) {
					if (dep == 0) break;
					if (! (first_loop && pre_picked)) //else: last node is null and is of an item picked above
						picked[stack[dep]] = false;
					dep--; first_loop = false;
				}
				if (dep == 0 && stack[dep] >= tbl.count() - 1) break;
				flag_back = true; //when flag_back become true, the next loop should goto its right sibling
			}
		}
		
		result.clear();
		for (unsigned int i = 0; i < tbl.count(); i++) {
			Item item(tbl[i].name(), pro[i]);
			result.item(item);
		}
	}
}
