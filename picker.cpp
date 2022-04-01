// by wuwbobo2021 <https://github.com/wuwbobo2021>, <wuwbobo@outlook.com>
// If you have found bugs in this program, please pull an issue, or contact me.

#include "picker.h"

using namespace RandomPicker;

static unsigned int determine(double val, const std::vector<double>& grid);
static bool included(unsigned int val, const std::vector<unsigned int>& vect);

void Picker::pick(unsigned int amount, std::vector<unsigned int>& result)
{
	if (amount == 0) return;
	if (!m_table.repetitive_picking && amount > m_table.count())
		throw std::invalid_argument("Picker::pick(): invalid amount.");
	
	std::vector<double> grid; double width;
	this->draw(grid); width = grid[m_table.count()]; if (width == 0) return;
	
	result.clear();
	double r; unsigned int n;
	for (unsigned int i = 0; i < amount; i++) {
		r = this->random_value(width);
		n = determine(r, grid);
		if (! m_table.repetitive_picking)
			if (included(n, result)) {i--; continue;}
		result.push_back(n);
	}
}

void Picker::pick(unsigned int amount, std::vector<std::string>& result)
{
	if (amount == 0) return;
	
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
	for (unsigned int i = 0; i < times; i++) {
		this->pick(amount, vect);
		for (unsigned int i = 0; i < amount; i++)
			stat[vect[i]]++;
	}
	
	result.clear();
	unsigned int cnt = m_table.count();
	for (unsigned int i = 0; i < cnt; i++) {
		Item item = Item(m_table[i].name(), 100.0 * (double)stat[i] / (times*amount));
		result.item(item);
	}
}

void Picker::draw(std::vector<double>& grid) const
{
	grid.clear();
	if (m_table.is_empty()) return;
	
	unsigned int cnt = m_table.count();
	double cur = 0;
	for (unsigned int i = 0; i < cnt; i++) {
		grid.push_back(cur);
		if (! m_table.power_inversed)
			cur += m_table[i].value();
		else {
			if (m_table[i].value() == 0) continue;
			cur += 1.0 / m_table[i].value();
		}
	}
	grid.push_back(cur);
}

static unsigned int determine(double val, const std::vector<double>& grid)
{
	unsigned int sz = grid.size();
	if (sz <= 2) return 0;
	
	const double* p = grid.data();
	for (unsigned int i = 0; i < sz - 1; i++) {
		if (val >= p[i] && val < p[i + 1]) return i;
		else if (i == sz - 2 && val == p[i + 1]) return i;
	}
	
	return 0; //it should be impossible
}

static bool included(unsigned int val, const std::vector<unsigned int>& vect)
{
	unsigned int sz = vect.size();
	if (sz == 0) return false;
	
	const unsigned int* p = vect.data();
	for (unsigned int i = 0; i < sz; i++)
		if (p[i] == val) return true;
	
	return false;
}

