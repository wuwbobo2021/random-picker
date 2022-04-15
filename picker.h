// by wuwbobo2021 <https://github.com/wuwbobo2021>, <wuwbobo@outlook.com>
// If you have found bugs in this program, please pull an issue, or contact me.

#ifndef RAMDOM_PICKER_PICKER_H
#define RAMDOM_PICKER_PICKER_H

#include "table.h"

#include <random>

namespace RandomPicker
{

class Picker
{
	Table& m_table;
	std::vector<double> m_grid;
	std::random_device m_ran_dev;
	bool m_flag_testing = false;
	
	double random_value(double width); //0.0 ~ width
	void draw();
	unsigned int determine(double val) const;
	
public:
	Picker(Table& table);
	void pick(unsigned int amount, std::vector<unsigned int>& result);
	void pick(unsigned int amount, std::vector<std::string>& result);
	void test(unsigned int times, unsigned int amount, Table& result);
	void calculate(unsigned int pick_amount, Table& result) const;
};

inline Picker::Picker(Table& table):
	m_table(table)
{}

inline double Picker::random_value(double width)
{
	unsigned int val = m_ran_dev();
	return width * (double)(val - m_ran_dev.min()) / (double)(m_ran_dev.max() - m_ran_dev.min());
}

}
#endif

